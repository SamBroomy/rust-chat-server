mod error;

use error::Result;
pub use error::ServerError;

use crate::{ClientMessage, Connection, FrameType, RoomManager, ServerMessage, User};

use crossterm::style::Stylize;

use std::collections::HashMap;

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::connection::Writer;

type RoomMap = Arc<Mutex<RoomManager>>;
type ClientMap = Arc<Mutex<HashMap<User, mpsc::Sender<(User, ServerMessage)>>>>;

pub struct Server {
    listener: TcpListener,
    clients: ClientMap,
    room_manager: RoomMap,
    server_broadcast_tx: broadcast::Sender<(User, ServerMessage)>,
}

impl Server {
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on: {}", listener.local_addr()?);
        // Create a broadcast channel to send messages to all clients
        let (broadcast_tx, _) = broadcast::channel(100);
        Ok(Self {
            listener,
            clients: Arc::new(Mutex::new(HashMap::default())),
            room_manager: Arc::new(Mutex::new(RoomManager::default())),
            server_broadcast_tx: broadcast_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Server started");
        loop {
            let (socket, _) = self.listener.accept().await?;
            info!("Accepted connection from: {:#}", socket.peer_addr()?);
            let clients = Arc::clone(&self.clients);
            let server_broadcast_tx = self.server_broadcast_tx.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, clients, server_broadcast_tx).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_client(
        socket: TcpStream,
        clients: ClientMap,
        server_broadcast_tx: broadcast::Sender<(User, ServerMessage)>,
    ) -> Result<()> {
        let (mut reader, mut writer) = Connection::split_owned(socket);

        debug!("Waiting for handshake frame");
        let user = match ClientMessage::read_frame_from(&mut reader).await? {
            ClientMessage::Handshake(user) => user,
            _ => return Err(ServerError::InvalidHandshake),
        };
        debug!("Handshake received: {}", user.username());

        // Create a channel to send messages to the client
        let (client_tx, mut client_rx) = mpsc::channel(16);
        {
            clients.lock().await.insert(user.clone(), client_tx);
        }

        // Send welcome message
        debug!("Sending welcome message to {}", user);
        ServerMessage::ServerMessage {
            content: format!("Welcome, {}!", user.username()),
        }
        .write_frame_to(&mut writer)
        .await?;
        // Subscribe to the broadcast channel
        let mut server_broadcast_rx = server_broadcast_tx.subscribe();

        debug!("Sending broadcast message");
        let welcome_frame = ServerMessage::ServerMessage {
            content: format!("{} joined the server", user.username().green()),
        };
        server_broadcast_tx.send((user.clone(), welcome_frame))?;

        loop {
            tokio::select! {
                Ok(frame) = ClientMessage::read_frame_from(&mut reader) => {
                    match frame {

                        ClientMessage::ChatMessage { content }=> {
                            Self::handle_chat_message(&user, content,&server_broadcast_tx)?;
                        }
                        ClientMessage::Ping(nonce) => {
                            Self::handle_ping(&user, nonce, &mut writer).await?;
                        }
                        ClientMessage::Disconnect => {
                            Self::handle_disconnect_request(&user, &server_broadcast_tx).await?;
                            break;
                        },
                        ClientMessage::ListRooms => {

                        }
                        ClientMessage::ListUsers => {
                            Self::handle_list_users().await?;
                        }
                        ClientMessage::Handshake(_) => return Err(ServerError::InvalidHandshake),
                        ClientMessage::Join { room:_ } => {
                            todo!()
                        }
                        ClientMessage::Create { room: _, description:_ } => {
                            todo!()
                        }
                        ClientMessage::Leave => {
                            todo!()
                        }
                        ClientMessage::PrivateMessage { to_user, content } => {
                            Self::handle_private_message(to_user, user.clone(), content, &clients, &mut writer).await?;
                        }

                        //_ => return Err(Error::ImplementFrame),
                    }
                }
                Ok((send_user ,message)) = server_broadcast_rx.recv() => {
                    if user != send_user {
                        info!("Sending from broadcast_rx");
                        message.write_frame_to(&mut writer).await?;
                    }
                }
                Some((send_user, message)) = client_rx.recv() => {
                    if user == send_user {
                        warn!("Sending back to original user");
                    }
                    info!("Sending from client_rx");
                    message.write_frame_to(&mut writer).await?;
                }
                //else => break,
            }
        }
        warn!(
            "Connection closed for {} due to breaking connection loop",
            user
        );
        clients.lock().await.remove(&user);
        Ok(())
    }

    fn handle_chat_message(
        user: &User,
        content: String,
        broadcast_tx: &broadcast::Sender<(User, ServerMessage)>,
    ) -> Result<()> {
        let message = ServerMessage::ChatMessage {
            from: user.clone(),
            content,
        };
        broadcast_tx.send((user.clone(), message))?;
        Ok(())
    }

    async fn handle_ping(user: &User, nonce: u64, writer: &mut Writer) -> Result<()> {
        debug!("Received ping frame from {}", user);
        ServerMessage::Pong(nonce).write_frame_to(writer).await?;
        Ok(())
    }

    async fn handle_disconnect_request(
        user: &User,
        broadcast_tx: &broadcast::Sender<(User, ServerMessage)>,
    ) -> Result<()> {
        debug!("Handling disconnect request from {}", user);
        let message = ServerMessage::ServerMessage {
            content: format!("{} disconnected", user.username().red()),
        };
        broadcast_tx.send((user.clone(), message))?;
        //clients.lock().await.remove(user);
        Ok(())
    }

    async fn handle_list_users() -> Result<()> {
        todo!();
        // let users = clients.lock().await.keys().cloned().collect();
        // let message = ServerFrame::ServerMessage {
        //     content: format!("Users: {:?}", users),
        // };
        // message.write_to(&mut writer).await?;
        // Ok(())
    }

    async fn handle_private_message(
        to_user: User,
        from_user: User,
        content: String,
        clients: &ClientMap,
        writer: &mut Writer,
    ) -> Result<()> {
        let message = ServerMessage::PrivateMessage {
            from: from_user.clone(),
            content,
        };
        match clients.lock().await.get(&to_user) {
            Some(tx) => {
                tx.send((from_user, message)).await?;
                info!("Sent private message to {:}", to_user);
                ServerMessage::ServerMessage {
                    content: format!("Private Message --> {}", to_user),
                }
                .write_frame_to(writer)
                .await?;

                Ok(())
            }
            None => {
                ServerMessage::Error {
                    message: "User not found".to_string(),
                }
                .write_frame_to(writer)
                .await?;
                error!("Failed to send private message to {}", to_user);
                Err(ServerError::UserNotFound(to_user.clone()))
            }
        }
    }
}
