use rdkafka::util::get_rdkafka_version;

use crate::consumer::holding::HoldingConsumer;
use crate::consumer::order::OrderConsumer;
use crate::consumer::trade::TradeConsumer;
use crate::consumer::transaction::TransactionConsumer;

mod consumer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (version_n, version_s) = get_rdkafka_version();
    println!("rdkafka version: 0x{:08x}, {}", version_n, version_s);

    let order_consumer = OrderConsumer::init();
    let holding_consumer = HoldingConsumer::init();
    let trade_consumer = TradeConsumer::init();
    let transaction_consumer = TransactionConsumer::init();

    let order_handle = tokio::spawn(order_consumer.listen());
    let holding_handle = tokio::spawn(holding_consumer.listen());
    let trades_handle = tokio::spawn(trade_consumer.listen());
    let transactions_handle = tokio::spawn(transaction_consumer.listen());

    let _ = tokio::join!(
        order_handle,
        holding_handle,
        trades_handle,
        transactions_handle
    );
}
