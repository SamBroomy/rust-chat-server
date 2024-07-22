use super::Result;
use crossterm::style::Stylize;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, instrument, warn};

use crate::common::{
    messages::{ServerInternal, ServerMessage, UserInternal, UserMessage},
    UserManager, UserName,
};

pub struct UserProcessor {
    user_processor_rx: mpsc::Receiver<UserMessage>,
    server_broadcast_tx: broadcast::Sender<ServerMessage>,
    user_manager: UserManager,
}

impl UserProcessor {
    pub fn new(
        user_processor_rx: mpsc::Receiver<UserMessage>,
        server_broadcast_tx: broadcast::Sender<ServerMessage>,
    ) -> Self {
        Self {
            user_processor_rx,
            server_broadcast_tx,
            user_manager: UserManager::default(),
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(user_message) = self.user_processor_rx.recv() => {
                    info!("User message: {:?}", user_message);
                    self.process_user_message(user_message).await?;
                }

            }
        }
    }

    #[instrument(skip_all, level = "debug")]
    async fn process_user_message(&mut self, user_message: UserMessage) -> Result<()> {
        let UserMessage { from_user, message } = user_message;
        match message {
            UserInternal::NewUser(sender) => {
                info!("New user: {}", from_user);
                let user_rx = self.user_manager.add_new_user(from_user.clone());
                sender.send(user_rx).unwrap();
                self.server_broadcast_tx.send(ServerMessage {
                    from_user: from_user.clone(),
                    content: ServerInternal::ServerMessage(
                        format!("{} joined the server", from_user.to_string().green()).to_string(),
                    ),
                })?;
            }
            UserInternal::GetUser(sender) => {
                info!("Get user info: {}", from_user);
                sender
                    .send(Ok(self.user_manager.get_user(&from_user)?.clone()))
                    .unwrap();
            }
            UserInternal::DisconnectUser => {
                info!("Disconnecting user: {}", from_user);
                match self.user_manager.remove_user(&from_user) {
                    Ok(_) => {
                        self.server_broadcast_tx.send(ServerMessage {
                            from_user: from_user.clone(),
                            content: ServerInternal::ServerMessage(format!(
                                "{} disconnected",
                                from_user
                            )),
                        })?;
                    }
                    Err(e) => {
                        warn!("Unable to disconnect user: {}", e);
                    }
                }
            }
            UserInternal::PrivateMessage { to_user, content } => {
                info!("Private message from: {} to: {}", from_user, to_user);

                match self.user_manager.get_user(&to_user) {
                    Ok(user) => {
                        let to_user_tx = user.user_tx();
                        if to_user == from_user {
                            to_user_tx
                                .send(ServerMessage {
                                    from_user,
                                    content: ServerInternal::Error(
                                        "You can't send a private message to yourself".to_string(),
                                    ),
                                })
                                .await?;
                        } else {
                            to_user_tx
                                .send(ServerMessage {
                                    from_user: from_user.clone(),
                                    content: ServerInternal::PrivateMessage { from_user, content },
                                })
                                .await?;
                        }
                    }
                    Err(e) => {
                        warn!("User does not exist: {e}");
                        if let Ok(from_user) = self.user_manager.get_user(&from_user) {
                            let from_user_name = from_user.user_name().clone();
                            from_user
                                .user_tx()
                                .send(ServerMessage {
                                    from_user: from_user_name,
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
                if let Ok(user) = self.user_manager.get_user(&from_user) {
                    user.user_tx()
                        .send(ServerMessage {
                            from_user,
                            content: ServerInternal::Pong(nonce),
                        })
                        .await?;
                }
            }
            UserInternal::ListUsers => {
                info!("List users from: {}", from_user);
                let users: Vec<UserName> = self.user_manager.list_users();
                if let Ok(user_tx) = self.user_manager.get_user(&from_user) {
                    user_tx
                        .user_tx()
                        .send(ServerMessage {
                            from_user,
                            content: ServerInternal::UserList { users },
                        })
                        .await?;
                }
            }
        }

        Ok(())
    }
}
