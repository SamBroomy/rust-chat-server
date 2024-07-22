use super::{ClientMessage, RoomMessage, ServerMessage, UserMessage};
use crate::common::UserName;

use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ProcessMessage {
    ClientMessage {
        from_user: UserName,
        message: ClientMessage,
    },
    ServerMessage {
        from_user: UserName,
        message: ServerMessage,
    },
    Internal(ProcessInternal),
}

#[derive(Debug)]
pub enum ProcessInternal {
    UserMessage(UserMessage),
    RoomMessage(RoomMessage),
    Response(ProcessResponse),
}

#[derive(Debug)]
pub enum ProcessResponse {
    UserCreated {
        username: UserName,
        user_rx: mpsc::Receiver<ServerMessage>,
    },
}
