use super::messages::{RoomInternal, RoomMessage, ServerInternal, UserInternal, UserMessage};
use super::User;
use super::{CommonError, Result};
use crate::common::messages::ServerMessage;
use crate::common::UserName;

use bincode::{Decode, Encode};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq, Hash)]
pub struct RoomName {
    room_name: String,
}

impl RoomName {
    pub fn new(room_name: impl Into<String>) -> Self {
        Self {
            room_name: room_name.into(),
        }
    }

    pub fn room_name(&self) -> &str {
        &self.room_name
    }
}

impl Display for RoomName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.room_name)
    }
}

impl PartialEq<&str> for RoomName {
    fn eq(&self, other: &&str) -> bool {
        self.room_name == *other
    }
}

impl PartialEq<String> for RoomName {
    fn eq(&self, other: &String) -> bool {
        self.room_name == *other
    }
}

impl From<String> for RoomName {
    fn from(name: String) -> Self {
        Self { room_name: name }
    }
}

impl From<&str> for RoomName {
    fn from(name: &str) -> Self {
        Self {
            room_name: name.to_string(),
        }
    }
}

pub struct RoomManager {
    room_name: RoomName,
    users: HashSet<User>,
    room_rx: mpsc::Receiver<RoomMessage>,
    user_processor_tx: mpsc::Sender<UserMessage>,
}

impl RoomManager {
    pub fn new(
        room_name: impl Into<RoomName>,
        user_processor_tx: Sender<UserMessage>,
    ) -> (Self, mpsc::Sender<RoomMessage>) {
        let (room_tx, room_rx) = mpsc::channel(32);
        (
            Self {
                room_name: room_name.into(),
                users: HashSet::new(),
                room_rx,
                user_processor_tx,
            },
            room_tx,
        )
    }

    async fn get_user_info(&mut self, from_user: UserName) -> Result<User> {
        let (user_info_tx, user_info_rx) = oneshot::channel();
        self.user_processor_tx
            .send(UserMessage {
                from_user,
                message: UserInternal::GetUser(user_info_tx),
            })
            .await?;
        user_info_rx.await.unwrap()
    }

    pub fn users_in_room(&self) -> bool {
        !self.users.is_empty()
    }

    pub fn room_name(&self) -> &RoomName {
        &self.room_name
    }

    pub fn list_users(&self) -> Vec<UserName> {
        self.users.iter().map(|u| u.user_name()).cloned().collect()
    }

    pub fn add_user(&mut self, user: User) -> Result<()> {
        if self.users.contains(&user) {
            return Err(CommonError::UserInRoom(user));
        }
        self.users.insert(user);
        Ok(())
    }

    pub fn remove_user(&mut self, user: &User) -> Result<()> {
        if self.users.remove(user) {
            Ok(())
        } else {
            Err(CommonError::UserNotInRoom(user.clone()))
        }
    }

    pub async fn send_room_message(
        &self,
        from_user: UserName,
        message: impl Into<String>,
    ) -> Result<()> {
        if self.users.is_empty() {
            return Err(CommonError::NoUsersInRoom);
        }

        let message = Arc::new(message.into());
        let from_user = Arc::new(from_user);
        let room_name = Arc::new(self.room_name().clone());

        let mut senders = FuturesUnordered::new();

        self.users.iter().for_each(|u| {
            let message = Arc::clone(&message);
            let from_user = Arc::clone(&from_user);
            let room_name = Arc::clone(&room_name);
            let user = u;

            senders.push(async move {
                user.user_tx()
                    .send(ServerMessage {
                        from_user: (*from_user).clone(),
                        content: ServerInternal::RoomMessage {
                            room: (*room_name).clone(),
                            from: (*from_user).clone(),
                            content: message.as_ref().clone(),
                        },
                    })
                    .await
            });
        });

        let mut errors = Vec::new();
        while let Some(result) = senders.next().await {
            if let Err(e) = result {
                warn!("Failed to send room message: {}", e);
                errors.push(e);
            }
        }
        if !errors.is_empty() {
            Err(CommonError::RoomMessageNotSent)
        } else {
            Ok(())
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(RoomMessage {
            from_user,
            room_name,
            message,
        }) = self.room_rx.recv().await
        {
            match message {
                RoomInternal::NewRoom | RoomInternal::ListRooms => {
                    // Do nothing as a new room is created by the room handler
                }
                RoomInternal::JoinRoom => {
                    let user = self.get_user_info(from_user.clone()).await?;

                    match self.add_user(user.clone()) {
                        Ok(_) => {
                            let message = format!("{} joined the room", user.user_name());
                            // Send room joined message to other users in the room
                            self.send_room_message(user.user_name().clone(), message)
                                .await?;
                        }
                        Err(e) => {
                            warn!("Failed to add user to room: {}", e);
                            user.user_tx()
                                .send(ServerMessage {
                                    from_user,
                                    content: ServerInternal::Error(e.to_string()),
                                })
                                .await?;
                        }
                    }
                }
                RoomInternal::LeaveRoom => {
                    let user = self.get_user_info(from_user.clone()).await?;
                    match self.remove_user(&user) {
                        Ok(_) => {
                            // Send room left message to other users in the room
                            let message = format!("{} left the room", user.user_name());
                            self.send_room_message(from_user, message).await?;
                        }
                        Err(e) => {
                            warn!("Failed to remove user from room: {}", e);
                            user.user_tx()
                                .send(ServerMessage {
                                    from_user,
                                    content: ServerInternal::Error(e.to_string()),
                                })
                                .await?;
                        }
                    }
                }
                RoomInternal::ListUsers => {
                    let user = self.get_user_info(from_user.clone()).await?;
                    let users = self.list_users();
                    user.user_tx()
                        .send(ServerMessage {
                            from_user,
                            content: ServerInternal::RoomUsers {
                                room: room_name,
                                users,
                            },
                        })
                        .await?;
                }
                RoomInternal::RoomMessage(content) => {
                    self.send_room_message(from_user, content).await?;
                }
            }
        }
        Ok(())
    }
}
