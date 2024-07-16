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

use crate::connection::OwnedWriter;

type RoomMap = Arc<Mutex<RoomManager>>;
type ClientMap = Arc<Mutex<HashMap<User, mpsc::Sender<(User, ServerMessage)>>>>;

#[derive(Debug, Default)]
pub struct Server {
    clients: ClientMap,
    room_manager: RoomMap,
    //server_broadcast_tx: broadcast::Sender<(User, ServerMessage)>,
}

impl Server {
    pub async fn run(&mut self, addr: impl ToSocketAddrs) -> Result<()> {
        info!("Server started");
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on: {}", listener.local_addr()?);

        // Create a broadcast channel to send messages to all clients
        let (server_broadcast_tx, _) = broadcast::channel(100);

        loop {
            let (socket, client_address) = listener.accept().await?;
            info!("Accepted connection from: {:#}", client_address);
            let clients = Arc::clone(&self.clients);
            let server_broadcast_tx = server_broadcast_tx.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, clients, server_broadcast_tx).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn authenticate(connection: &mut Connection) -> Result<User> {
        debug!("Waiting for handshake frame");
        let user = match connection.read_frame().await? {
            ClientMessage::Handshake(user) => user,
            _ => return Err(ServerError::InvalidHandshake),
        };
        debug!("Handshake received: {}", user);
        // Send back a response to the client
        connection
            .write_frame(&ServerMessage::ServerMessage(format!(
                "Welcome, {}!",
                user.to_string().green()
            )))
            .await?;
        debug!("Handshake complete");
        Ok(user)
    }

    async fn handle_client(
        socket: TcpStream,
        clients: ClientMap,
        server_broadcast_tx: broadcast::Sender<(User, ServerMessage)>,
    ) -> Result<()> {
        let mut connection = Connection::from_stream(socket);

        let user = Self::authenticate(&mut connection).await?;

        let (mut reader, mut writer) = connection.split_into();

        // Create a channel to send messages to the client
        let (client_tx, mut client_rx) = mpsc::channel(16);
        {
            clients.lock().await.insert(user.clone(), client_tx);
        }

        // Subscribe to the broadcast channel
        let mut server_broadcast_rx = server_broadcast_tx.subscribe();

        debug!("Sending broadcast message");
        let welcome_frame =
            ServerMessage::ServerMessage(format!("{} joined the server", user.username().green()));
        server_broadcast_tx.send((user.clone(), welcome_frame))?;

        loop {
            tokio::select! {
                Ok(frame) = ClientMessage::read_frame_from(&mut reader) => {
                    match frame {

                        ClientMessage::ChatMessage (content )=> {
                            Self::handle_chat_message(&user, content, &server_broadcast_tx)?;
                        }
                        ClientMessage::Ping(nonce) => {
                            Self::handle_ping(&user, nonce, &mut writer).await?;
                        }
                        ClientMessage::Disconnect => {
                            Self::handle_disconnect_request(&user, &clients, &server_broadcast_tx).await?;
                            break;
                        },
                        ClientMessage::ListRooms => {
                            todo!()
                        }
                        ClientMessage::ListUsers => {
                            todo!()
                        }
                        ClientMessage::Handshake(_) => return Err(ServerError::InvalidHandshake),
                        ClientMessage::Join (room) => {
                            todo!()
                        }
                        ClientMessage::CreateRoom (room) => {
                            todo!()
                        }
                        ClientMessage::Leave => {
                            todo!()
                        }
                        ClientMessage::PrivateMessage { to_user, content } => {
                            Self::handle_private_message(to_user, user.clone(), content, &clients, &mut writer).await?;
                        }
                    }
                }
                Ok((send_user ,message)) = server_broadcast_rx.recv() => {
                    if user != send_user {
                        info!("Sending from server_broadcast_rx");
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
                else => break,
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

    async fn handle_ping(user: &User, nonce: u16, writer: &mut OwnedWriter) -> Result<()> {
        debug!("Received ping frame from {}", user);
        ServerMessage::Pong(nonce).write_frame_to(writer).await?;
        Ok(())
    }

    async fn handle_disconnect_request(
        user: &User,
        clients: &ClientMap,
        broadcast_tx: &broadcast::Sender<(User, ServerMessage)>,
    ) -> Result<()> {
        debug!("Handling disconnect request from {}", user);
        let message =
            ServerMessage::ServerMessage(format!("{} disconnected", user.username().red()));
        broadcast_tx.send((user.clone(), message))?;
        clients.lock().await.remove(user);
        Ok(())
    }

    async fn handle_private_message(
        to_user: User,
        from_user: User,
        content: String,
        clients: &ClientMap,
        writer: &mut OwnedWriter,
    ) -> Result<()> {
        let message = ServerMessage::PrivateMessage {
            from: from_user.clone(),
            content,
        };
        match clients.lock().await.get(&to_user) {
            Some(tx) => {
                ServerMessage::ServerMessage(format!(
                    "{} {}",
                    "Private Message -->".cyan(),
                    to_user.to_string().yellow()
                ))
                .write_frame_to(writer)
                .await?;
                tx.send((from_user, message)).await?;
                info!("Sent private message to {:}", to_user);
                Ok(())
            }
            None => {
                ServerMessage::Error(format!("User not found: {}", to_user))
                    .write_frame_to(writer)
                    .await?;
                error!("Failed to send private message to {}", to_user);
                Err(ServerError::UserNotFound(to_user.clone()))
            }
        }
    }
}
