use chat_app::{init, Result, Server};

use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = init(Level::TRACE);

    let mut server = Server::new(addr).await?;

    Ok(server.run().await?)
}
