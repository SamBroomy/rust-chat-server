use super::Result;
use crate::common::{
    messages::{
        ClientMessage, ProcessInternal, ProcessMessage, RoomInternal, RoomMessage, ServerInternal,
        ServerMessage, UserInternal, UserMessage,
    },
    UserName,
};

use tokio::sync::{broadcast, mpsc};
use tracing::{info, instrument, warn};

pub struct ServerProcessor {
    server_processor_rx: mpsc::Receiver<ProcessMessage>,
    user_processor_tx: mpsc::Sender<UserMessage>,
    room_processor_tx: mpsc::Sender<RoomMessage>,
    server_broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl ServerProcessor {
    pub fn new(
        server_processor_rx: mpsc::Receiver<ProcessMessage>,
        user_processor_tx: mpsc::Sender<UserMessage>,
        room_processor_tx: mpsc::Sender<RoomMessage>,
        server_broadcast_tx: broadcast::Sender<ServerMessage>,
    ) -> Self {
        Self {
            server_processor_rx,
            user_processor_tx,
            room_processor_tx,
            server_broadcast_tx,
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn run(&mut self) -> Result<()> {
        // Start a new task to handle room messages

        while let Some(message) = self.server_processor_rx.recv().await {
            match message {
                ProcessMessage::Internal(process_internal) => {
                    // Start new task
                    self.handle_internal_message(process_internal).await?;
                }
                ProcessMessage::ClientMessage { from_user, message } => {
                    self.handle_client_message(from_user, message).await?;
                }
                ProcessMessage::ServerMessage { from_user, message } => {
                    warn!("Received server message from {}", from_user);
                    self.handle_server_message(from_user, message).await?;
                }
            }
            info!("Received server command");
        }
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    async fn handle_internal_message(&mut self, process_internal: ProcessInternal) -> Result<()> {
        match process_internal {
            ProcessInternal::UserMessage(user_message) => {
                warn!("Received user message: {:?}", user_message);
                self.user_processor_tx.send(user_message).await?;
            }
            ProcessInternal::Response(response) => {
                warn!("Received response: {:?}", response)
            }
            ProcessInternal::RoomMessage(room_message) => {
                warn!("Received room message: {:?}", room_message);
                self.room_processor_tx.send(room_message).await?;
            }
        }
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    async fn handle_client_message(
        &mut self,
        from_user: UserName,
        message: ClientMessage,
    ) -> Result<()> {
        info!("Received client message from {}: {:?}", from_user, message);

        match message {
            ClientMessage::Disconnect => {
                // TODO: Remove from any room
                self.handle_internal_message(ProcessInternal::UserMessage(UserMessage {
                    from_user,
                    message: UserInternal::DisconnectUser,
                }))
                .await?;
            }
            ClientMessage::GlobalChatMessage(content) => {
                self.server_broadcast_tx.send(ServerMessage {
                    from_user: from_user.clone(),
                    content: ServerInternal::GlobalChatMessage { from_user, content },
                })?;
            }
            ClientMessage::PrivateMessage { to_user, content } => {
                self.handle_internal_message(ProcessInternal::UserMessage(UserMessage {
                    from_user,
                    message: UserInternal::PrivateMessage { to_user, content },
                }))
                .await?;
            }
            ClientMessage::Ping(nonce) => {
                self.handle_internal_message(ProcessInternal::UserMessage(UserMessage {
                    from_user,
                    message: UserInternal::Ping(nonce),
                }))
                .await?;
            }
            ClientMessage::ListUsers => {
                self.handle_internal_message(ProcessInternal::UserMessage(UserMessage {
                    from_user,
                    message: UserInternal::ListUsers,
                }))
                .await?;
            }
            ClientMessage::CreateRoom(room) => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: room,
                    message: RoomInternal::NewRoom,
                }))
                .await?;
            }
            ClientMessage::JoinRoom(room) => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: room,
                    message: RoomInternal::JoinRoom,
                }))
                .await?;
            }
            ClientMessage::LeaveRoom(room) => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: room,
                    message: RoomInternal::LeaveRoom,
                }))
                .await?;
            }
            ClientMessage::ListRooms => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: "N/A".into(),
                    message: RoomInternal::ListRooms,
                }))
                .await?;
            }
            ClientMessage::ListRoomUsers(room) => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: room,
                    message: RoomInternal::ListUsers,
                }))
                .await?;
            }
            ClientMessage::RoomMessage { room, content } => {
                self.handle_internal_message(ProcessInternal::RoomMessage(RoomMessage {
                    from_user,
                    room_name: room,
                    message: RoomInternal::RoomMessage(content),
                }))
                .await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self), name = "Server", level = "warn")]
    async fn handle_server_message(
        &mut self,
        from_user: UserName,
        message: ServerMessage,
    ) -> Result<()> {
        info!("Received server message from {}: {:?}", from_user, message);
        self.server_broadcast_tx.send(message)?;
        Ok(())
    }
}
