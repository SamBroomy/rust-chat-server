mod client;
mod connection;
mod error;
mod frame;
mod server;

pub use client::Client;
pub use connection::Connection;
pub use error::{Error, Result};
pub use frame::{Frame, FrameType};
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        //.with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .init();
    // New tcp listener bound to localhost:8080
    let addr = "localhost:8080";
    let mut server = Server::new(addr).await?;

    return server.run().await;
}
