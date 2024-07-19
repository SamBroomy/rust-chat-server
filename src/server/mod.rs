mod client_handler;
mod error;
mod processor;
mod user_handler;

use crate::common::messages::ServerMessage;
use client_handler::ClientHandler;
use error::Result;
pub use error::ServerError;
use processor::ServerProcessor;
use user_handler::UserProcessor;

use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

#[derive(Debug)]
pub struct Server {
    server_broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl Default for Server {
    fn default() -> Self {
        // Create a broadcast channel: Used to send server messages to all threads.
        let (server_broadcast_tx, _) = broadcast::channel(32);
        Self {
            server_broadcast_tx,
        }
    }
}

impl Server {
    pub async fn run(&mut self, addr: impl ToSocketAddrs) -> Result<()> {
        info!("Server started");
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on: {}", listener.local_addr()?);

        // Start a new task to handle users
        let (user_processor_tx, user_processor_rx) = mpsc::channel(32);
        let (server_processor_tx, server_processor_rx) = mpsc::channel(64);

        let user_processor = UserProcessor::new(
            user_processor_rx,
            server_processor_tx.clone(),
            self.server_broadcast_tx.clone(),
        );

        tokio::spawn(async move {
            if let Err(e) = user_processor.run().await {
                error!("Error running user processor: {}", e);
            }
        });

        let mut server_processor = ServerProcessor::new(
            server_processor_rx,
            user_processor_tx,
            self.server_broadcast_tx.clone(),
        );

        // Spawn the server processor to handle server commands and take that processing task away from client connections.
        tokio::spawn(async move {
            if let Err(e) = server_processor.run().await {
                error!("Error running server processor: {}", e);
            }
        });

        loop {
            let (socket, client_address) = listener.accept().await?;
            info!("Accepted connection from: {:#}", client_address);

            let mut handler = match ClientHandler::init(
                socket,
                self.server_broadcast_tx.subscribe(),
                server_processor_tx.clone(),
            )
            .await
            {
                Ok(handler) => handler,
                Err(e) => {
                    error!("Error initializing client handler: {}", e);
                    continue;
                }
            };

            tokio::spawn(async move {
                if let Err(e) = handler.run().await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }
}
