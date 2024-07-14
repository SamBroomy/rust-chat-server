use chat_app::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        //.with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::WARN)
        .init();
    println!("Enter your username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim();
    let address = "localhost:8080";

    let client = Client::new(address, username).await?;

    client.run().await
}
