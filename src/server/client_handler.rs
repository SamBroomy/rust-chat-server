use std::net::SocketAddr;

use super::Result;

use crossterm::style::Stylize;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
};
use tracing::{debug, info, warn};

use crate::{
    common::ProcessResponse, server::ServerError, ClientMessage, Connection, FrameType,
    ProcessMessage, ServerMessage, UserName,
};

/// Handles the client connection, reading and writing messages to the stream.
pub struct ClientHandler {
    socket_addr: SocketAddr,
    user: UserName,
    connection: Connection,
    server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
    server_command_tx: mpsc::Sender<ProcessMessage>,
    client_rx: mpsc::Receiver<ProcessMessage>,
}

impl ClientHandler {
    pub async fn init(
        socket_addr: SocketAddr,
        connection: TcpStream,
        server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
        mut server_command_tx: mpsc::Sender<ProcessMessage>,
    ) -> Result<Self> {
        let mut connection = Connection::from_stream(connection);
        let (client_tx, mut client_rx) = mpsc::channel(16);

        let user = Self::authenticate(
            &mut connection,
            &mut server_command_tx,
            client_tx,
            &mut client_rx,
        )
        .await?;

        Ok(Self {
            socket_addr,
            user,
            connection,
            server_broadcast_tx,
            server_command_tx,
            client_rx,
        })
    }

    async fn authenticate(
        connection: &mut Connection,
        server_command_tx: &mut mpsc::Sender<ProcessMessage>,
        client_tx: mpsc::Sender<ProcessMessage>,
        client_rx: &mut mpsc::Receiver<ProcessMessage>,
    ) -> Result<UserName> {
        debug!("Waiting for handshake frame");
        let user = match connection.read_frame().await? {
            ClientMessage::Handshake(user) => user,
            _ => return Err(ServerError::InvalidHandshake),
        };

        debug!("Handshake received: {}", user);

        debug!("Add user to server");
        // Send the user to the server processor
        server_command_tx
            .send(ProcessMessage::NewUser(user.clone(), client_tx))
            .await?;

        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                connection.write_frame(&ServerMessage::Error("Handshake timeout".to_string())).await?;
                connection.close().await;
                return Err(ServerError::HandshakeTimeout);
            }
            Some(process_message) = client_rx.recv() => {
                match process_message {
                    ProcessMessage::Response(ProcessResponse::Complete) => {
                        debug!("User added to server");
                    }
                    _ => return Err(ServerError::InvalidHandshake),
                }
            }


        }

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

    pub async fn run(&mut self) -> Result<()> {
        let (mut reader, mut writer) = self.connection.split();

        // TODO:: Should be part of the process message.
        debug!("Sending broadcast message");
        let welcome_frame = ServerMessage::ServerMessage(format!(
            "{} joined the server",
            self.user.username().green()
        ));
        // Subscribe to the broadcast channel
        let mut server_broadcast_rx = self.server_broadcast_tx.subscribe();
        self.server_broadcast_tx
            .send((self.user.clone(), welcome_frame))?;
        debug!("Broadcast message sent");

        loop {
            tokio::select! {
            Ok(frame) = ClientMessage::read_frame_from(&mut reader) => {
                let message = ProcessMessage::ClientMessage {
                    from_user: self.user.clone(),
                    message: frame,
                };
                self.server_command_tx.send(message).await?;

                /*
                            server_command_tx.send((current_user.clone(), frame.clone())).await?;
                            match frame {

                                ClientMessage::ChatMessage (content )=> {
                                    Self::handle_chat_message(&current_user, content, &server_broadcast_tx)?;
                                }
                                ClientMessage::RoomMessage { room, content } => {
                                    Self::handle_room_message(&current_user, room, content, &rooms).await?;
                                }
                                ClientMessage::Ping(nonce) => {
                                    Self::handle_ping(&current_user, nonce, &mut writer).await?;
                                }
                                ClientMessage::Disconnect => {
                                    Self::handle_disconnect_request(&current_user, &clients, &server_broadcast_tx).await?;
                                    break;
                                },
                                ClientMessage::ListRooms => {
                                    Self::handle_list_rooms(&current_user, &rooms, &mut writer).await?;
                                }
                                ClientMessage::ListUsers => {
                                    Self::handle_list_users(&current_user, &clients, &mut writer).await?;
                                }
                                ClientMessage::Join (room) => {
                                    Self::join_room(&current_user, room, &rooms, &server_command_tx).await?;
                                }
                                ClientMessage::Handshake(_) => return Err(ServerError::InvalidHandshake),
                                ClientMessage::CreateRoom (room) => {
                                    Self::handle_create_room(&current_user, room,  &rooms, &server_command_tx).await?;
                                }
                                ClientMessage::Leave => {
                                    todo!()
                                }
                                ClientMessage::PrivateMessage { to_user, content } => {
                                    Self::handle_private_message(to_user, current_user.clone(), content, &clients, &mut writer).await?;
                                }
                                */

            }

            Ok((send_user ,message)) = server_broadcast_rx.recv() => {
                if self.user != send_user {
                    info!("Sending from server_broadcast_rx");
                    message.write_frame_to(&mut writer).await?;
                }
            }

            Some(message) = self.client_rx.recv() => {
                match message {
                ProcessMessage::ClientMessage { from_user, message } => {
                    if self.user == from_user {
                        debug!("Message from self");
                    }
                    info!("Sending from client_rx send user: {} current user: {}", from_user, self.user);
                    message.write_frame_to(&mut writer).await?;
                }
                _ => {
                    warn!("Received message from client_rx that is not a ClientMessage");
                    break;
                }
            }}}
        }

        warn!(
            "Connection closed for {} due to breaking connection loop",
            self.user
        );

        Ok(())
    }
}
