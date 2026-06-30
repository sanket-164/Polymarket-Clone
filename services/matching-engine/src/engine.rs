use crate::nats_handler::NatsHandler;
use common::model::{FeedMessage, Order, OrderFeed, OrderSide, TradeMessage};
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

    async fn publish_messages(nats_handler: &NatsHandler, buy: Order, sell: Order) {
        let filled = buy.remaining_shares.min(sell.remaining_shares);
        println!("Trade: {} shares for {}", filled, sell.price);

        for order in [buy.clone(), sell.clone()] {
            let feed_message = FeedMessage::OrderFeed {
                feed: OrderFeed {
                    market_id: order.market_id,
                    outcome_id: order.outcome_id,
                    side: order.side,
                    quantity: -filled, // negative to signal reduction to feed subscribers
                    price: order.price,
                },
            };

            if let Err(e) = nats_handler.feed_market_order(feed_message).await {
                eprintln!("Failed to publish feed update OrderFeed message: {:?}", e);
            }
        }

        if let Err(e) = nats_handler
            .trade_update_order(TradeMessage::UpdateOrders { buy, sell })
            .await
        {
            eprintln!("Failed to publish trade UpdateOrder message: {:?}", e);
        }
    }

    pub async fn match_order(&mut self, order: Order, nats_handler: &NatsHandler) {
        let Some(book) = self.get_order_book_mut(&order.market_id, &order.outcome_id) else {
            return;
        };

        match order.side {
            OrderSide::BUY => {
                let mut new_buy = order;

                loop {
                    match book.peek_sell() {
                        Some(best_sell) if new_buy.price >= best_sell.price => {
                            let mut sell = book.pop_sell().unwrap();

                            Engine::publish_messages(nats_handler, new_buy.clone(), sell.clone())
                                .await;

                            match new_buy.remaining_shares.cmp(&sell.remaining_shares) {
                                Ordering::Greater => {
                                    new_buy.remaining_shares -= sell.remaining_shares;
                                }
                                Ordering::Less => {
                                    sell.remaining_shares -= new_buy.remaining_shares;
                                    book.push_sell(sell.clone());
                                    break;
                                }
                                Ordering::Equal => {
                                    break;
                                }
                            }
                        }

                        // No match — unmatched remainder stays on book (already in Redis from place_order).
                        _ => {
                            book.push_buy(new_buy);
                            break;
                        }
                    }
                }
            }

            OrderSide::SELL => {
                let mut new_sell = order;

                loop {
                    match book.peek_buy() {
                        Some(best_buy) if new_sell.price <= best_buy.price => {
                            let mut buy = book.pop_buy().unwrap();

                            Engine::publish_messages(nats_handler, buy.clone(), new_sell.clone())
                                .await;

                            match new_sell.remaining_shares.cmp(&buy.remaining_shares) {
                                Ordering::Greater => {
                                    new_sell.remaining_shares -= buy.remaining_shares;
                                }
                                Ordering::Less => {
                                    buy.remaining_shares -= new_sell.remaining_shares;
                                    book.push_buy(buy.clone());
                                    break;
                                }
                                Ordering::Equal => {
                                    break;
                                }
                            }
                        }
                        // No match — unmatched remainder stays on book (already in Redis from place_order).
                        _ => {
                            book.push_sell(new_sell);
                            break;
                        }
                    }
                }
            }
        }
    }
}
