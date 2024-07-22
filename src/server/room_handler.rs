use std::collections::HashMap;

use super::Result;

use crate::common::{
    messages::{
        RoomInternal, RoomMessage, ServerInternal, ServerMessage, UserInternal, UserMessage,
    },
    RoomManager, RoomName, User, UserName,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::info;

pub struct RoomProcessor {
    room_processor_rx: mpsc::Receiver<RoomMessage>,
    user_processor_tx: mpsc::Sender<UserMessage>,
    server_broadcast_tx: broadcast::Sender<ServerMessage>,
    room_manager: HashMap<RoomName, mpsc::Sender<RoomMessage>>,
}

impl RoomProcessor {
    pub fn new(
        room_processor_rx: mpsc::Receiver<RoomMessage>,
        user_processor_tx: mpsc::Sender<UserMessage>,
        server_broadcast_tx: broadcast::Sender<ServerMessage>,
    ) -> Self {
        Self {
            room_processor_rx,
            user_processor_tx,
            server_broadcast_tx,
            room_manager: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(RoomMessage {
            from_user,
            room_name,
            message,
        }) = self.room_processor_rx.recv().await
        {
            match message {
                RoomInternal::NewRoom => {
                    info!("New room: {}", from_user);
                    let (room_manager, room_tx) =
                        RoomManager::new(room_name.clone(), self.user_processor_tx.clone());
                    self.room_manager.insert(room_name.clone(), room_tx);

                    tokio::spawn(async move {
                        if let Err(e) = room_manager.run().await {
                            info!("Error running room manager: {}", e);
                        }
                    });
                    self.notify_user(
                        from_user.clone(),
                        format!("You created room: {}", room_name).to_string(),
                    )
                    .await?;

                    self.server_broadcast_tx.send(ServerMessage {
                        from_user,
                        content: ServerInternal::ServerMessage(
                            format!("Room {} created", room_name).to_string(),
                        ),
                    })?;
                }
                RoomInternal::ListRooms => {
                    info!("List rooms: {}", from_user);
                    let rooms: String = self
                        .room_manager
                        .keys()
                        .map(|room_name| room_name.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    self.notify_user(
                        from_user.clone(),
                        format!("Rooms: [{:}]", rooms).to_string(),
                    )
                    .await?;
                }
                RoomInternal::JoinRoom
                | RoomInternal::LeaveRoom
                | RoomInternal::ListUsers
                | RoomInternal::RoomMessage(_) => {
                    if let Some(room_tx) = self.room_manager.get(&room_name) {
                        room_tx
                            .send(RoomMessage {
                                from_user,
                                room_name,
                                message,
                            })
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn get_user_info(&mut self, from_user: UserName) -> Result<User> {
        let (user_info_tx, user_info_rx) = oneshot::channel();
        self.user_processor_tx
            .send(UserMessage {
                from_user,
                message: UserInternal::GetUser(user_info_tx),
            })
            .await?;
        Ok(user_info_rx.await??)
    }

    async fn notify_user(&mut self, from_user: UserName, message: String) -> Result<()> {
        self.get_user_info(from_user.clone())
            .await?
            .user_tx()
            .send(ServerMessage {
                from_user: from_user.clone(),
                content: ServerInternal::ServerMessage(message),
            })
            .await?;

        Ok(())
    }
}
