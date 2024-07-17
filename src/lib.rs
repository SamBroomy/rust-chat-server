mod client;
mod common;
mod connection;
mod error;

mod server;

pub use client::Client;
pub use common::{
    ClientMessage, ProcessInternal, ProcessMessage, RoomManager, RoomName, ServerMessage, UserName,
};
pub use connection::{Connection, FrameType};
pub use error::{Error, Result};
pub use server::Server;
use tracing::{level_filters::LevelFilter, warn};

/// Initialize the logger and read the .env file to get the address
pub fn init(log_level: impl TryInto<LevelFilter>) -> String {
    setup_tracing(log_level);
    get_address_from_env()
}

fn setup_tracing(log_level: impl TryInto<LevelFilter>) {
    let log_level = log_level.try_into().unwrap_or_else(|_| {
        warn!("Invalid log level, using default: WARN");
        LevelFilter::WARN
    });
    tracing_subscriber::fmt()
        //.with_span_events(FmtSpan::CLOSE)
        .with_max_level(log_level)
        .init();
}

fn get_address_from_env() -> String {
    dotenv::dotenv().ok();

    let address = std::env::var("HOST").unwrap_or_else(|_| {
        warn!("HOST env var not set!! Using default: localhost");
        "localhost".to_string()
    });
    let port = std::env::var("PORT").unwrap_or_else(|_| {
        warn!("PORT env var not set!! Using default: 8080");
        "8080".to_string()
    });
    format!("{}:{}", address, port)
}
