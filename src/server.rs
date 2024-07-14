use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

use crate::{Connection, Error, Frame, Result};

type ClientMap = Arc<Mutex<HashMap<String, mpsc::Sender<Frame>>>>;

pub struct Server {
    listener: TcpListener,
    clients: ClientMap,
    broadcast_tx: broadcast::Sender<(String, Frame)>,
}

impl Server {
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on: {}", listener.local_addr()?);
        let (broadcast_tx, _) = broadcast::channel(100);
        Ok(Self {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Server started");
        loop {
            let (socket, _) = self.listener.accept().await?;
            info!("Accepted connection from: {:#}", socket.peer_addr()?);
            let clients = Arc::clone(&self.clients);
            let broadcast_tx = self.broadcast_tx.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, clients, broadcast_tx).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        clients: ClientMap,
        broadcast_tx: broadcast::Sender<(String, Frame)>,
    ) -> Result<()> {
        let mut connection = Connection::new(socket);

        debug!("Waiting for handshake frame");
        let username = match connection.read_frame().await? {
            Frame::Handshake { username } => username,
            _ => return Err(Error::InvalidHandshake),
        };
        debug!("Handshake received: {}", username);

        // Add client to the map
        let (client_tx, mut client_rx) = mpsc::channel(100);
        {
            clients.lock().await.insert(username.clone(), client_tx);
        }

        // Send welcome message
        debug!("Sending welcome message to {}", username);
        connection
            .write_frame(&Frame::ServerMessage {
                content: format!("Welcome, {}!", username),
            })
            .await?;

        let mut broadcast_rx = broadcast_tx.subscribe();

        loop {
            debug!("Waiting for frame from {}", username);
            tokio::select! {
                Ok(frame) = connection.read_frame() => {
                    match frame {
                        Frame::ChatMessage { content, .. } => {
                            let message = Frame::ChatMessage {
                                username: username.clone(),
                                content,
                            };
                            broadcast_tx.send((username.clone(), message))?;
                        }
                        Frame::Ping(nonce) => {
                            connection.write_frame(&Frame::Pong(nonce)).await?;
                        }
                        _ =>
                        return Err(Error::ImplementFrame),
                    }
                }
                Ok((other_username ,message)) = broadcast_rx.recv() => {
                    if username != other_username {
                        info!("Sending from broadcast_rx");
                        connection.write_frame(&message).await?;
                    }

                }
                Some(message) = client_rx.recv() => {
                    info!("Sending from client_rx");
                    connection.write_frame(&message).await?;
                }
                else => break,
            }
        }

        clients.lock().await.remove(&username);
        Ok(())
    }
}
