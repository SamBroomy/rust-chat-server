use super::Result;
use crossterm::style::Stylize;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, instrument};

use crate::common::{
    messages::{ProcessMessage, ServerInternal, ServerMessage, UserInternal, UserMessage},
    UserName,
};

pub struct UserProcessor {
    user_processor_rx: mpsc::Receiver<UserMessage>,
    server_command_tx: mpsc::Sender<ProcessMessage>,
    server_broadcast_tx: broadcast::Sender<ServerMessage>,
    users: HashMap<UserName, mpsc::Sender<ServerMessage>>,
}

impl UserProcessor {
    pub fn new(
        user_processor_rx: mpsc::Receiver<UserMessage>,
        server_command_tx: mpsc::Sender<ProcessMessage>,
        server_broadcast_tx: broadcast::Sender<ServerMessage>,
    ) -> Self {
        Self {
            user_processor_rx,
            server_command_tx,
            server_broadcast_tx,
            users: HashMap::new(),
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn run(mut self) -> Result<()> {
        while let Some(UserMessage { from_user, message }) = self.user_processor_rx.recv().await {
            match message {
                UserInternal::NewUser(sender) => {
                    info!("New user: {}", from_user);
                    let (user_tx, user_rx) = mpsc::channel(32);
                    self.users.insert(from_user.clone(), user_tx.clone());
                    sender.send(user_rx).unwrap();
                    self.server_broadcast_tx.send(ServerMessage {
                        from_user: from_user.clone(),
                        content: ServerInternal::ServerMessage(
                            format!("{} joined the server", from_user.to_string().green())
                                .to_string(),
                        ),
                    })?;
                }
                UserInternal::DisconnectUser => {
                    info!("Disconnecting user: {}", from_user);
                    self.users.remove(&from_user);
                    self.server_broadcast_tx.send(ServerMessage {
                        from_user: from_user.clone(),
                        content: ServerInternal::ServerMessage(format!(
                            "{} disconnected",
                            from_user
                        )),
                    })?;
                }
                UserInternal::PrivateMessage { to_user, content } => {
                    info!("Private message from: {} to: {}", from_user, to_user);
                    match self.users.get(&to_user) {
                        Some(to_user_tx) => {
                            if to_user == from_user {
                                to_user_tx
                                    .send(ServerMessage {
                                        from_user,
                                        content: ServerInternal::Error(
                                            "You can't send a private message to yourself"
                                                .to_string(),
                                        ),
                                    })
                                    .await?;
                            } else {
                                to_user_tx
                                    .send(ServerMessage {
                                        from_user: from_user.clone(),
                                        content: ServerInternal::PrivateMessage {
                                            from_user,
                                            content,
                                        },
                                    })
                                    .await?;
                            }
                        }
                        None => {
                            if let Some(from_user_tx) = self.users.get(&from_user) {
                                from_user_tx
                                    .send(ServerMessage {
                                        from_user,
                                        content: ServerInternal::Error(format!(
                                            "User not found: {}",
                                            to_user
                                        )),
                                    })
                                    .await?;
                            };
                        }
                    }
                }
                UserInternal::Ping(nonce) => {
                    info!("Ping from: {}", from_user);
                    if let Some(user_tx) = self.users.get(&from_user) {
                        user_tx
                            .send(ServerMessage {
                                from_user,
                                content: ServerInternal::Pong(nonce),
                            })
                            .await?;
                    }
                }
                UserInternal::ListUsers => {
                    info!("List users from: {}", from_user);
                    let users: Vec<UserName> = self.users.keys().cloned().collect();
                    if let Some(user_tx) = self.users.get(&from_user) {
                        user_tx
                            .send(ServerMessage {
                                from_user,
                                content: ServerInternal::UserList { users },
                            })
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
