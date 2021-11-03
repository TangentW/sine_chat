use sine_chat;

const ADDR: &str = "127.0.0.1:8888";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Running ...");
    sine_chat::run_server(&ADDR).await
}
