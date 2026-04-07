mod nats_consumer;

use nats_consumer::Consumer;

#[tokio::main]
async fn main() {
    let consumer = match Consumer::new("nats://localhost:4222").await {
        Ok(c) => c,
        _ => {
            println!("Failed to connect consumer");
            std::process::exit(1);
        }
    };

    match consumer.consume("TASKS").await {
        Err(e) => {
            println!("{e}")
        }
        _ => {}
    }
}
