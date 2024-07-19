use crate::common::{messages::ServerMessage, UserName};

use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct UserMessage {
    pub from_user: UserName,
    pub message: UserInternal,
}

#[derive(Debug)]
pub enum UserInternal {
    NewUser(oneshot::Sender<mpsc::Receiver<ServerMessage>>),
    PrivateMessage { to_user: UserName, content: String },
    DisconnectUser,
    Ping(u16),
    ListUsers,
}
