mod engine;

use common::{
    config::NatsConfig, constant::MATCHER_STREAM, model::MatcherMessage, nats_handler::NatsHandler,
};
use futures::StreamExt;

use crate::engine::Engine;

#[tokio::main]
async fn main() {
    let nats_handler = match NatsHandler::new(&NatsConfig::init().nats_url).await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect nats handler: {e}");
            std::process::exit(1);
        }
    };

    let mut message_stream = nats_handler
        .get_message_stream(MATCHER_STREAM)
        .await
        .expect("Failed to get messages");

    let mut engine = Engine::new();

    println!("Matcher is Ready!");

    while let Some(msg) = message_stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Message error: {e}");
                continue;
            }
        };

        let message: MatcherMessage = match serde_json::from_slice(&msg.payload) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Deserialize error: {e}");
                let _ = msg.ack().await;
                continue;
            }
        };

        match message {
            MatcherMessage::PlaceOrder { order } => {
                engine.match_order(order, &nats_handler).await;
            }
            MatcherMessage::CreateMarket { market, outcomes } => {
                engine.add_market(
                    market.id,
                    outcomes.first_outcome.id,
                    outcomes.second_outcome.id,
                );
            }
            MatcherMessage::RemoveMarket { market_id } => {
                engine.remove_market(market_id);
            }
            MatcherMessage::CancelOrder { order: _order } => {
                // handle cancel order
            }
        }

        if let Err(e) = msg.ack().await {
            eprintln!("Ack failed: {e}");
        }
    }
}
