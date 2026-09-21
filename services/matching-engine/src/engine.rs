use crate::nats_handler::NatsHandler;
use chrono::{DateTime, Utc};
use common::model::{Order, OrderSide, OrderType, TradeMessage};
use rust_decimal::Decimal;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::println;
use uuid::Uuid;

#[derive(Debug)]
struct OrderBook {
    // Buy orders:
    // BTreeMap sorts ascending. We want highest price first.
    // So we use Reverse<Decimal> for the price key.
    buy: BTreeMap<Reverse<Decimal>, VecDeque<Order>>,

    // Sell orders:
    // BTreeMap sorts ascending. Lowest price first.
    sell: BTreeMap<Decimal, VecDeque<Order>>,

    // Fast lookup for cancellations!
    index: HashMap<Uuid, (bool, Decimal)>, // (is_buy, price)
}

impl OrderBook {
    fn new() -> Self {
        Self {
            buy: BTreeMap::new(),
            sell: BTreeMap::new(),
            index: HashMap::new(),
        }
    }

    // Add a buy order at the end of the queue for a price level. If the price level doesn't exist, create it.
    fn add_buy(&mut self, order: Order) {
        self.index.insert(order.id, (true, order.price));
        self.buy
            .entry(Reverse(order.price))
            .or_default()
            .push_back(order);
    }
    // Add a sell order at the end of the queue for a price level. If the price level doesn't exist, create it.
    fn add_sell(&mut self, order: Order) {
        self.index.insert(order.id, (false, order.price));
        self.sell.entry(order.price).or_default().push_back(order);
    }

    // Get the best buy order (highest price, earliest timestamp) without removing it from the book.
    fn best_buy(&self) -> Option<&Order> {
        self.buy
            .iter()
            .find(|(_, orders)| !orders.is_empty())
            .and_then(|(_, orders)| orders.front())
    }

    // Get the best sell order (lowest price, earliest timestamp) without removing it from the book.
    fn best_sell(&self) -> Option<&Order> {
        self.sell
            .iter()
            .find(|(_, orders)| !orders.is_empty())
            .and_then(|(_, orders)| orders.front())
    }

    // Allows in-place mutation to preserve queue priority
    fn best_buy_mut(&mut self) -> Option<&mut Order> {
        self.buy
            .iter_mut()
            .find(|(_, orders)| !orders.is_empty())
            .and_then(|(_, orders)| orders.front_mut())
    }

    // Allows in-place mutation to preserve queue priority
    fn best_sell_mut(&mut self) -> Option<&mut Order> {
        self.sell
            .iter_mut()
            .find(|(_, orders)| !orders.is_empty())
            .and_then(|(_, orders)| orders.front_mut())
    }

    // Remove the best buy order (highest price, earliest timestamp) from the book and return it.
    fn remove_buy(&mut self) -> Option<Order> {
        let (&Reverse(price), orders) = self.buy.iter_mut().find(|(_, o)| !o.is_empty())?;
        let order = orders.pop_front()?;
        if orders.is_empty() {
            self.buy.remove(&Reverse(price));
        }
        self.index.remove(&order.id);
        Some(order)
    }

    // Remove the best sell order (lowest price, earliest timestamp) from the book and return it.
    fn remove_sell(&mut self) -> Option<Order> {
        let (&price, orders) = self.sell.iter_mut().find(|(_, o)| !o.is_empty())?;
        let order = orders.pop_front()?;
        if orders.is_empty() {
            self.sell.remove(&price);
        }
        self.index.remove(&order.id);
        Some(order)
    }

    // Clean up expired orders cleanly
    fn remove_expired_best_buy(&mut self, now: DateTime<Utc>) -> Option<Order> {
        if let Some(best) = self.best_buy() {
            if best.expires_at <= now {
                return self.remove_buy();
            }
        }
        None
    }

    fn remove_expired_best_sell(&mut self, now: DateTime<Utc>) -> Option<Order> {
        if let Some(best) = self.best_sell() {
            if best.expires_at <= now {
                return self.remove_sell();
            }
        }
        None
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

        println!(
            "Added market {} with outcomes {} and {}",
            market_id, first_outcome_id, second_outcome_id
        );
    }

    pub fn remove_market(&mut self, market_id: Uuid) {
        self.order_books.remove(&market_id);

        println!("Removed market {}", market_id);
    }

    fn get_order_book_mut(
        &mut self,
        market_id: &Uuid,
        outcome_id: &Uuid,
    ) -> Option<&mut OrderBook> {
        self.order_books.get_mut(market_id)?.get_mut(outcome_id)
    }

    async fn publish_limit_trade(nats_handler: &NatsHandler, buy: Order, sell: Order) {
        let filled = buy.remaining_shares.min(sell.remaining_shares);
        println!("Trade: {} shares for {}", filled, sell.price);

        if let Err(e) = nats_handler
            .trade_limit_order(TradeMessage::LimitOrders {
                buy,
                sell,
                timestamp: Utc::now().timestamp_millis(),
            })
            .await
        {
            eprintln!("Failed to publish trade UpdateOrder message: {:?}", e);
        }
    }

    async fn publish_market_trade(
        nats_handler: &NatsHandler,
        market_order: Order,
        book_order: Order,
    ) {
        let filled = match market_order.side {
            OrderSide::BUY => {
                if market_order.quote_amount >= book_order.price * book_order.remaining_shares {
                    book_order.remaining_shares
                } else {
                    (market_order.quote_amount / book_order.price).floor()
                }
            }
            OrderSide::SELL => market_order
                .remaining_shares
                .min(book_order.remaining_shares),
        };

        println!("Trade: {} shares for {}", filled, book_order.price);

        if let Err(e) = nats_handler
            .trade_market_order(TradeMessage::MarketOrders {
                market_order,
                book_order,
                timestamp: Utc::now().timestamp_millis(),
            })
            .await
        {
            eprintln!("Failed to publish trade UpdateOrder message: {:?}", e);
        }
    }

    async fn limit_order(&mut self, order: Order, nats_handler: &NatsHandler) {
        let Some(book) = self.get_order_book_mut(&order.market_id, &order.outcome_id) else {
            return;
        };

        match order.side {
            OrderSide::BUY => {
                let mut new_buy = order;

                loop {
                    let now = Utc::now();

                    // Clean up any expired best sell orders before attempting to match
                    while let Some(expired) = book.remove_expired_best_sell(now) {
                        println!("Sell order expired: {:?}", expired);
                    }

                    // Determine if we have a match. The immutable borrow ends at the semicolon.
                    let is_match = book
                        .best_sell()
                        .map_or(false, |best| new_buy.price >= best.price);

                    if is_match {
                        // Clone the order for the trade message.
                        // The immutable borrow of `book` ends immediately after this line.
                        let matched_sell = book.best_sell().cloned().unwrap();
                        let matched_buy = new_buy.clone();

                        Engine::publish_limit_trade(
                            nats_handler,
                            matched_buy,
                            matched_sell.clone(),
                        )
                        .await;

                        // Handle fill scenarios using mutable borrows (now perfectly safe)
                        if new_buy.remaining_shares > matched_sell.remaining_shares {
                            // Full fill of resting order, incoming order still has shares
                            let _ = book.remove_sell();
                            new_buy.remaining_shares -= matched_sell.remaining_shares;
                            // Loop continues to match against the next best sell
                        } else if new_buy.remaining_shares == matched_sell.remaining_shares {
                            // Full fill of both orders
                            let _ = book.remove_sell();
                            break;
                        } else {
                            // PARTIAL FILL of resting order: Mutate in place to preserve queue priority!
                            if let Some(best_sell_mut) = book.best_sell_mut() {
                                best_sell_mut.remaining_shares -= new_buy.remaining_shares;
                            }
                            break; // Incoming order is completely filled, exit loop
                        }
                    } else {
                        // No match — unmatched remainder stays on book
                        book.add_buy(new_buy);
                        break;
                    }
                }
            }

            OrderSide::SELL => {
                let mut new_sell = order;

                loop {
                    let now = Utc::now();

                    // Clean up any expired best buy orders before attempting to match
                    while let Some(expired) = book.remove_expired_best_buy(now) {
                        println!("Buy order expired: {:?}", expired);
                    }

                    // Determine if we have a match. The immutable borrow ends at the semicolon.
                    let is_match = book
                        .best_buy()
                        .map_or(false, |best| new_sell.price <= best.price);

                    if is_match {
                        // Clone the order for the trade message.
                        // The immutable borrow of `book` ends immediately after this line.
                        let matched_buy = book.best_buy().cloned().unwrap();
                        let matched_sell = new_sell.clone();

                        Engine::publish_limit_trade(
                            nats_handler,
                            matched_buy.clone(),
                            matched_sell,
                        )
                        .await;

                        // Handle fill scenarios using mutable borrows (now perfectly safe)
                        if new_sell.remaining_shares > matched_buy.remaining_shares {
                            // Full fill of resting order, incoming order still has shares
                            let _ = book.remove_buy();
                            new_sell.remaining_shares -= matched_buy.remaining_shares;
                        } else if new_sell.remaining_shares == matched_buy.remaining_shares {
                            // Full fill of both orders
                            let _ = book.remove_buy();
                            break;
                        } else {
                            // PARTIAL FILL of resting order: Mutate in place to preserve queue priority!
                            if let Some(best_buy_mut) = book.best_buy_mut() {
                                best_buy_mut.remaining_shares -= new_sell.remaining_shares;
                            }
                            break; // Incoming order is completely filled, exit loop
                        }
                    } else {
                        // No match — unmatched remainder stays on book
                        book.add_sell(new_sell);
                        break;
                    }
                }
            }
        }
    }

    pub async fn cancel_order(&mut self, order: Order, nats_handler: &NatsHandler) {
        let Some(book) = self.get_order_book_mut(&order.market_id, &order.outcome_id) else {
            return;
        };

        let Some((is_buy, price)) = book.index.remove(&order.id) else {
            return;
        };

        let orders = if is_buy {
            book.buy.get_mut(&Reverse(price))
        } else {
            book.sell.get_mut(&price)
        };

        if let Some(orders) = orders {
            let initial_len = orders.len();

            orders.retain(|o| o.id != order.id);

            // If length decreased, the order was successfully found and removed
            if orders.len() < initial_len {
                // Clean up the price level from the BTreeMap if it's now empty
                if orders.is_empty() {
                    if is_buy {
                        book.buy.remove(&Reverse(price));
                    } else {
                        book.sell.remove(&price);
                    }
                }

                nats_handler
                    .trade_cancel_order(TradeMessage::CancelOrder {
                        order,
                        timestamp: Utc::now().timestamp_millis(),
                    })
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to publish trade CancelOrder message: {:?}", e);
                    });
            }
        }
    }

    async fn market_order(&mut self, order: Order, nats_handler: &NatsHandler) {
        let Some(book) = self.get_order_book_mut(&order.market_id, &order.outcome_id) else {
            return;
        };

        match order.side {
            OrderSide::BUY => {
                let mut new_buy = order;

                loop {
                    let now = Utc::now();

                    // Clean up any expired best sell orders before attempting to match
                    while let Some(expired) = book.remove_expired_best_sell(now) {
                        println!("Sell order expired: {:?}", expired);
                    }

                    let is_match = book
                        .best_sell()
                        .map_or(false, |best| new_buy.quote_amount >= best.price);

                    if is_match {
                        // Clone the order for the trade message.
                        // The immutable borrow of `book` ends immediately after this line.
                        let matched_sell = book.best_sell().cloned().unwrap();

                        Engine::publish_market_trade(
                            nats_handler,
                            new_buy.clone(),
                            matched_sell.clone(),
                        )
                        .await;

                        let sell_cost = matched_sell.price * matched_sell.remaining_shares;

                        // Full fill of resting order, incoming order still has shares
                        if new_buy.quote_amount > sell_cost {
                            new_buy.quote_amount -= sell_cost;
                            let _ = book.remove_sell();

                        // Full fill of both orders
                        } else if new_buy.quote_amount == sell_cost {
                            let _ = book.remove_sell();
                            break;

                        // PARTIAL FILL of resting order: Mutate in place to preserve queue priority!
                        } else {
                            if let Some(best_sell_mut) = book.best_sell_mut() {
                                best_sell_mut.remaining_shares -=
                                    (new_buy.quote_amount / best_sell_mut.price).floor();
                            }

                            // Incoming order is completely filled, exit loop
                            break;
                        }
                    } else {
                        nats_handler
                            .trade_complete_order(TradeMessage::CompleteOrder {
                                market_order: new_buy,
                            })
                            .await
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to publish trade CompleteOrder message: {:?}", e);
                            });
                        break;
                    }
                }
            }

            OrderSide::SELL => {
                let mut new_sell = order;

                loop {
                    let now = Utc::now();

                    // Clean up any expired best buy orders before attempting to match
                    while let Some(expired) = book.remove_expired_best_buy(now) {
                        println!("Buy order expired: {:?}", expired);
                    }

                    // Determine if we have a match. The immutable borrow ends at the semicolon.
                    let is_match = book.best_buy().is_some();

                    if is_match {
                        // Clone the order for the trade message.
                        // The immutable borrow of `book` ends immediately after this line.
                        let matched_buy = book.best_buy().cloned().unwrap();

                        Engine::publish_market_trade(
                            nats_handler,
                            new_sell.clone(),
                            matched_buy.clone(),
                        )
                        .await;

                        // Full fill of resting order, incoming order still has shares
                        if new_sell.remaining_shares > matched_buy.remaining_shares {
                            new_sell.remaining_shares -= matched_buy.remaining_shares;
                            let _ = book.remove_buy();

                        // Full fill of both orders
                        } else if new_sell.remaining_shares == matched_buy.remaining_shares {
                            let _ = book.remove_buy();
                            break;

                        // PARTIAL FILL of resting order: Mutate in place to preserve queue priority!
                        } else {
                            if let Some(best_buy_mut) = book.best_buy_mut() {
                                best_buy_mut.remaining_shares -= new_sell.remaining_shares;
                            }

                            // Incoming order is completely filled, exit loop
                            break;
                        }
                    } else {
                        nats_handler
                            .trade_complete_order(TradeMessage::CompleteOrder {
                                market_order: new_sell,
                            })
                            .await
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to publish trade CompleteOrder message: {:?}", e);
                            });
                        break;
                    }
                }
            }
        }
    }

    pub async fn match_order(&mut self, order: Order, nats_handler: &NatsHandler) {
        match order.order_type {
            OrderType::LIMIT => {
                self.limit_order(order, nats_handler).await;
            }
            OrderType::MARKET => {
                self.market_order(order, nats_handler).await;
            }
        }
    }
}
