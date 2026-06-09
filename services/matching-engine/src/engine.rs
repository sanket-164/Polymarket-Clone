use crate::db::TradeExt;
use common::database::client::PGClient;
use common::model::{FeedMessage, Order, OrderFeed, OrderType};
use common::nats_handler::NatsHandler;
use deadpool_redis::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq)]
struct BuyOrder(pub Order);

#[derive(Debug, Eq, PartialEq)]
struct SellOrder(pub Order);

// Buy orders: min-heap, lowest price first
impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse price comparison
        other
            .0
            .price
            .cmp(&self.0.price)
            .then_with(|| other.0.created_at.cmp(&self.0.created_at))
    }
}

impl PartialOrd for SellOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Buy orders: max-heap, highest price first
impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .price
            .cmp(&other.0.price)
            .then_with(|| other.0.created_at.cmp(&self.0.created_at))
    }
}

impl PartialOrd for BuyOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct OrderBook {
    buy: BinaryHeap<BuyOrder>,
    sell: BinaryHeap<SellOrder>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            buy: BinaryHeap::new(),
            sell: BinaryHeap::new(),
        }
    }

    fn push_buy(&mut self, order: Order) {
        self.buy.push(BuyOrder(order));
    }

    fn peek_buy(&mut self) -> Option<&Order> {
        self.buy.peek().map(|b| &b.0)
    }

    fn pop_buy(&mut self) -> Option<Order> {
        self.buy.pop().map(|b| b.0)
    }

    fn push_sell(&mut self, order: Order) {
        self.sell.push(SellOrder(order));
    }

    fn peek_sell(&self) -> Option<&Order> {
        self.sell.peek().map(|s| &s.0)
    }

    fn pop_sell(&mut self) -> Option<Order> {
        self.sell.pop().map(|s| s.0)
    }
}

#[derive(Debug)]
pub struct Engine {
    order_books: HashMap<Uuid, HashMap<Uuid, OrderBook>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            order_books: HashMap::new(),
        }
    }

    pub fn add_market(&mut self, market_id: Uuid, first_outcome_id: Uuid, second_outcome_id: Uuid) {
        if self.order_books.contains_key(&market_id) {
            return;
        }

        let outcome_books = self
            .order_books
            .entry(market_id)
            .or_insert_with(HashMap::new);
        outcome_books.insert(first_outcome_id, OrderBook::new());
        outcome_books.insert(second_outcome_id, OrderBook::new());
    }

    pub fn remove_market(&mut self, market_id: Uuid) {
        self.order_books.remove(&market_id);
    }

    fn get_order_book_mut(
        &mut self,
        market_id: &Uuid,
        outcome_id: &Uuid,
    ) -> Option<&mut OrderBook> {
        self.order_books.get_mut(market_id)?.get_mut(outcome_id)
    }

    async fn uprem_price(
        redis: &mut Connection,
        market_id: &Uuid,
        outcome_id: &Uuid,
        order_type: &str,
        price: &Decimal,
        filled_shares: Decimal,
    ) {
        let base_key = format!("orderbook:{}:{}:{}", market_id, outcome_id, order_type);
        let qty_key = format!("{}:qty", base_key);
        let price_str = price.normalize().to_string();

        // Update the shares in HashMap
        let new_qty: f64 = match redis::cmd("HINCRBYFLOAT")
            .arg(&qty_key)
            .arg(&price_str)
            .arg(-filled_shares.to_f64().unwrap_or(0.0))
            .query_async(redis)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Redis HINCRBYFLOAT failed: {:?}", e);
                return;
            }
        };

        // Remove price & shares from HashMap & SortedSet if share's quantity is 0
        if new_qty <= 0.0 {
            if let Err(e) = redis::pipe()
                .cmd("HDEL")
                .arg(&qty_key)
                .arg(&price_str)
                .cmd("ZREM")
                .arg(&base_key)
                .arg(&price_str)
                .query_async::<()>(redis)
                .await
            {
                eprintln!("Redis cleanup failed: {:?}", e);
            }
        }
    }

    async fn publish_feed(
        nats_handler: &NatsHandler,
        market_id: Uuid,
        outcome_id: Uuid,
        price: Decimal,
        // Negative value = shares were consumed (reduction)
        delta: Decimal,
    ) {
        let feed_message = FeedMessage {
            order: Some(OrderFeed {
                market_id,
                outcome_id,
                quantity: -delta, // negative to signal reduction to feed subscribers
                price,
            }),
            market_id: None,
        };

        if let Err(e) = nats_handler.feed_market_order(feed_message).await {
            eprintln!("Failed to publish feed message: {:?}", e);
        }
    }

    pub async fn match_order(
        &mut self,
        order: Order,
        pg_client: &PGClient,
        redis: &mut Connection,
        nats_handler: &NatsHandler,
    ) {
        let Some(book) = self.get_order_book_mut(&order.market_id, &order.outcome_id) else {
            return;
        };

        match order.order_type {
            OrderType::BUY => {
                let mut remaining = order;

                loop {
                    match book.peek_sell() {
                        Some(best_sell) if remaining.price >= best_sell.price => {
                            let mut sell = book.pop_sell().unwrap();
                            let filled = remaining.remaining_shares.min(sell.remaining_shares);

                            if let Err(e) = pg_client.trade(remaining.clone(), sell.clone()).await {
                                eprintln!("Trade failed: {:?}", e);
                                book.push_sell(sell);
                                book.push_buy(remaining);
                                break;
                            }

                            println!("Trade {} -> {}", remaining.id, sell.id);

                            Engine::uprem_price(
                                redis,
                                &remaining.market_id,
                                &remaining.outcome_id,
                                "buy",
                                &remaining.price,
                                filled,
                            )
                            .await;
                            Engine::uprem_price(
                                redis,
                                &sell.market_id,
                                &sell.outcome_id,
                                "sell",
                                &sell.price,
                                filled,
                            )
                            .await;

                            // Publish feed for both sides
                            Engine::publish_feed(
                                nats_handler,
                                remaining.market_id,
                                remaining.outcome_id,
                                remaining.price,
                                filled,
                            )
                            .await;
                            Engine::publish_feed(
                                nats_handler,
                                sell.market_id,
                                sell.outcome_id,
                                sell.price,
                                filled,
                            )
                            .await;

                            match remaining.remaining_shares.cmp(&sell.remaining_shares) {
                                Ordering::Greater => {
                                    remaining.remaining_shares -= sell.remaining_shares;
                                }
                                Ordering::Less => {
                                    sell.remaining_shares -= remaining.remaining_shares;
                                    book.push_sell(sell);
                                    break;
                                }
                                Ordering::Equal => break,
                            }
                        }
                        // No match — unmatched remainder stays on book (already in Redis from place_order).
                        _ => {
                            book.push_buy(remaining);
                            break;
                        }
                    }
                }
            }

            OrderType::SELL => {
                let mut remaining = order;

                loop {
                    match book.peek_buy() {
                        Some(best_buy) if remaining.price <= best_buy.price => {
                            let mut buy = book.pop_buy().unwrap();
                            let filled = remaining.remaining_shares.min(buy.remaining_shares);

                            if let Err(e) = pg_client.trade(buy.clone(), remaining.clone()).await {
                                eprintln!("Trade failed: {:?}", e);
                                book.push_buy(buy);
                                book.push_sell(remaining);
                                break;
                            }

                            println!("Trade {} -> {}", buy.id, remaining.id);

                            // Update Redis for both sides of the trade.
                            Engine::uprem_price(
                                redis,
                                &remaining.market_id,
                                &remaining.outcome_id,
                                "sell",
                                &remaining.price,
                                filled,
                            )
                            .await;
                            Engine::uprem_price(
                                redis,
                                &buy.market_id,
                                &buy.outcome_id,
                                "buy",
                                &buy.price,
                                filled,
                            )
                            .await;

                            // Publish feed for both sides
                            Engine::publish_feed(
                                nats_handler,
                                remaining.market_id,
                                remaining.outcome_id,
                                remaining.price,
                                filled,
                            )
                            .await;
                            Engine::publish_feed(
                                nats_handler,
                                buy.market_id,
                                buy.outcome_id,
                                buy.price,
                                filled,
                            )
                            .await;

                            match remaining.remaining_shares.cmp(&buy.remaining_shares) {
                                Ordering::Greater => {
                                    remaining.remaining_shares -= buy.remaining_shares;
                                }
                                Ordering::Less => {
                                    buy.remaining_shares -= remaining.remaining_shares;
                                    book.push_buy(buy);
                                    break;
                                }
                                Ordering::Equal => break,
                            }
                        }
                        // No match — unmatched remainder stays on book (already in Redis from place_order).
                        _ => {
                            book.push_sell(remaining);
                            break;
                        }
                    }
                }
            }
        }
    }
}
