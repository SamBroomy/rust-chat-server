mod client_handler;
mod error;
mod processor;

use client_handler::ClientHandler;
use error::Result;
pub use error::ServerError;
use processor::ServerProcessor;

use crate::{ServerMessage, UserName};

use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

#[derive(Debug)]
pub struct Server {
    server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
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

        // Create a channel to send server commands to the server to process and handle.
        let (server_command_tx, server_command_rx) = mpsc::channel(64);

        let mut server_processor =
            ServerProcessor::new(server_command_rx, self.server_broadcast_tx.clone());
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
                client_address,
                socket,
                self.server_broadcast_tx.clone(),
                server_command_tx.clone(),
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
