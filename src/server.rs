use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

use crate::{ClientFrame, Connection, Error, FrameType, Result, ServerFrame, User};

type ClientMap = Arc<Mutex<HashMap<User, mpsc::Sender<ServerFrame>>>>;

pub struct Server {
    listener: TcpListener,
    clients: ClientMap,
    broadcast_tx: broadcast::Sender<(User, ServerFrame)>,
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
        mut socket: TcpStream,
        clients: ClientMap,
        broadcast_tx: broadcast::Sender<(User, ServerFrame)>,
    ) -> Result<()> {
        let (mut reader, mut writer) = Connection::split(&mut socket);

        debug!("Waiting for handshake frame");
        let user = match ClientFrame::read_from(&mut reader).await? {
            ClientFrame::Handshake(user) => user,
            _ => return Err(Error::InvalidHandshake),
        };
        debug!("Handshake received: {}", user.username());

        // Add client to the map
        let (client_tx, mut client_rx) = mpsc::channel(16);
        debug!("Adding client to the map");
        {
            clients.lock().await.insert(user.clone(), client_tx);
        }

        // Send welcome message
        debug!("Sending welcome message to {}", user);
        ServerFrame::ServerMessage {
            content: format!("Welcome, {}!", user.username()),
        }
        .write_to(&mut writer)
        .await?;

        let mut broadcast_rx = broadcast_tx.subscribe();

        loop {
            tokio::select! {
                Ok(frame) = ClientFrame::read_from(&mut reader) => {
                    match frame {
                        ClientFrame::ChatMessage { content }=> {
                            let message = ServerFrame::ChatMessage {
                                user: user.clone(),
                                content,
                            };
                            broadcast_tx.send((user.clone(), message))?;
                        }
                        ClientFrame::Ping(nonce) => {
                            ServerFrame::Pong(nonce).write_to(&mut writer).await?;
                        }
                        ClientFrame::Disconnect => {
                            ServerFrame::ServerMessage {
                                content: format!("{} requested disconnect", user.username()),
                            }.write_to(&mut writer).await?;
                            break;
                        },
                        _ => return Err(Error::ImplementFrame),
                    }
                    //parse_frame(frame, &username, &clients, &broadcast_tx).await?;

                }
                Ok((send_user ,message)) = broadcast_rx.recv() => {
                    if user != send_user {
                        info!("Sending from broadcast_rx");
                        message.write_to(&mut writer).await?;
                    }

                }
                Some(message) = client_rx.recv() => {
                    info!("Sending from client_rx");
                    message.write_to(&mut writer).await?;
                }
                else => break,
            }
        }

        clients.lock().await.remove(&user);
        Ok(())
    }
}
