use super::Result;
use crate::{common::ProcessResponse, ProcessMessage, RoomName, ServerMessage, UserName};

use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

pub struct ServerProcessor {
    server_command_rx: mpsc::Receiver<ProcessMessage>,
    server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
    users: HashMap<UserName, mpsc::Sender<ProcessMessage>>,
    rooms: HashMap<RoomName, mpsc::Sender<(UserName, ServerMessage)>>,
}

impl ServerProcessor {
    pub fn new(
        server_command_rx: mpsc::Receiver<ProcessMessage>,
        server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
    ) -> Self {
        Self {
            server_command_rx,
            server_broadcast_tx,
            users: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.server_command_rx.recv().await {
            match message {
                ProcessMessage::NewUser(user, tx) => {
                    info!("New user: {}", user);
                    self.users.insert(user.clone(), tx.clone());
                    tx.send(ProcessMessage::Response(ProcessResponse::Complete))
                        .await?;
                }
                _ => {
                    todo!("Handle other messages: {:?}", message);
                }
            }
            info!("Received server command");
        }
        Ok(())
    }
}

/*

    async fn handle_client(
        socket: TcpStream,
        clients: ClientMap,
        rooms: RoomMap,
        server_broadcast_tx: broadcast::Sender<(UserName, ServerMessage)>,
        server_command_tx: mpsc::Sender<(UserName, ClientMessage)>,
    ) -> Result<()> {
        let mut connection = Connection::from_stream(socket);

        let current_user = Self::authenticate(&mut connection).await?;

        let (mut reader, mut writer) = connection.split_into();

        // Create a channel to send messages to the client
        let (client_tx, mut client_rx) = mpsc::channel(16);
        {
            clients.lock().await.insert(current_user.clone(), client_tx);
        }

        // Subscribe to the broadcast channel
        let mut server_broadcast_rx = server_broadcast_tx.subscribe();

        debug!("Sending broadcast message");
        let welcome_frame = ServerMessage::ServerMessage(format!(
            "{} joined the server",
            current_user.username().green()
        ));
        server_broadcast_tx.send((current_user.clone(), welcome_frame))?;

        loop {
            tokio::select! {
                Ok(frame) = ClientMessage::read_frame_from(&mut reader) => {



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
                    }
                }
                Ok((send_user ,message)) = server_broadcast_rx.recv() => {
                    if current_user != send_user {
                        info!("Sending from server_broadcast_rx");
                        message.write_frame_to(&mut writer).await?;
                    }
                }
                Some((send_user, message)) = client_rx.recv() => {
                    if current_user == send_user {
                        debug!("Message from self");
                        //continue;
                    }
                    info!("Sending from client_rx send user: {} current user: {}", send_user, current_user);
                    message.write_frame_to(&mut writer).await?;
                }
                else => break,
            }
        }
        warn!(
            "Connection closed for {} due to breaking connection loop",
            current_user
        );
        clients.lock().await.remove(&current_user);
        Ok(())
    }

    fn handle_chat_message(
        user: &UserName,
        content: String,
        broadcast_tx: &broadcast::Sender<(UserName, ServerMessage)>,
    ) -> Result<()> {
        let message = ServerMessage::ChatMessage {
            from: user.clone(),
            content,
        };
        broadcast_tx.send((user.clone(), message))?;
        Ok(())
    }

    async fn handle_ping(user: &UserName, nonce: u16, writer: &mut OwnedWriter) -> Result<()> {
        debug!("Received ping frame from {}", user);
        ServerMessage::Pong(nonce).write_frame_to(writer).await?;
        Ok(())
    }

    async fn handle_disconnect_request(
        user: &UserName,
        clients: &ClientMap,
        broadcast_tx: &broadcast::Sender<(UserName, ServerMessage)>,
    ) -> Result<()> {
        debug!("Handling disconnect request from {}", user);
        let message =
            ServerMessage::ServerMessage(format!("{} disconnected", user.username().red()));
        broadcast_tx.send((user.clone(), message))?;
        clients.lock().await.remove(user);
        Ok(())
    }

    async fn handle_private_message(
        to_user: UserName,
        from_user: UserName,
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

    async fn handle_create_room(
        user: &UserName,
        room: RoomName,
        rooms: &RoomMap,
        command_tx: &mpsc::Sender<(UserName, ServerMessage)>,
    ) -> Result<()> {
        debug!("Handling create room request from {}", user);
        let mut room_manager = rooms.lock().await;
        match room_manager.create_room(room.clone()) {
            Ok(room) => {
                let message = ServerMessage::RoomCreated(room.clone());
                command_tx.send((user.clone(), message)).await?;
                info!("Room created: {}", room);
                Ok(())
            }
            Err(e) => {
                let message = ServerMessage::Error(format!("Failed to create room: {}", e));
                command_tx.send((user.clone(), message)).await?;
                error!("Failed to create room: {}", e);
                Err(e.into())
            }
        }
    }

    async fn handle_list_rooms(
        user: &UserName,
        rooms: &RoomMap,
        writer: &mut OwnedWriter,
    ) -> Result<()> {
        debug!("Handling list rooms request from {}", user);
        let room_manager = rooms.lock().await;
        let rooms = room_manager.list_rooms();
        let message = ServerMessage::RoomList {
            rooms: rooms.clone(),
        };
        message.write_frame_to(writer).await?;
        info!("Sent room list to {}", user);
        Ok(())
    }

    async fn handle_list_users(
        user: &UserName,
        clients: &ClientMap,
        writer: &mut OwnedWriter,
    ) -> Result<()> {
        debug!("Handling list users request from {}", user);
        let clients = clients.lock().await;
        let users = clients.keys().cloned().collect();
        let message = ServerMessage::UserList { users };
        message.write_frame_to(writer).await?;
        info!("Sent user list to {}", user);
        Ok(())
    }

    async fn join_room(
        user: &UserName,
        room: RoomName,
        rooms: &RoomMap,
        command_tx: &mpsc::Sender<(UserName, ServerMessage)>,
    ) -> Result<()> {
        debug!("Handling join room request from {}", user);
        let room_manager = rooms.lock().await;
        match room_manager.join_room(&room, user) {
            Ok(rx) => {
                let message = ServerMessage::RoomJoined {
                    room: room.clone(),
                    user: user.clone(),
                };
                command_tx.send((user.clone(), message)).await?;
                info!("User {} joined room {}", user, room);
                Ok(())
            }
            Err(e) => {
                let message = ServerMessage::Error(format!("Failed to join room: {}", e));
                command_tx.send((user.clone(), message)).await?;
                error!("Failed to join room: {}", e);
                Err(e.into())
            }
        }
    }

    async fn handle_room_message(
        user: &UserName,
        room: RoomName,
        content: String,
        rooms: &RoomMap,
    ) -> Result<()> {
        debug!("Handling room message from {}", user);
        let room_manager = rooms.lock().await;
        match room_manager.send_message(
            &room,
            user,
            ServerMessage::RoomMessage {
                room: room.clone(),
                from: user.clone(),
                content: content.clone(),
            },
        ) {
            Ok(()) => {
                info!("Sent message to room {}: {}", room, content);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send message to room {}: {}", room, e);
                Err(e.into())
            }
        }
    }
*/
