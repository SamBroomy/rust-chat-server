use tokio::sync::mpsc;

use crate::{ClientMessage, ServerMessage, UserName};

/// Messages sent by the client to the server
#[derive(Debug, Clone)]
pub enum ProcessMessage {
    ClientMessage {
        from_user: UserName,
        message: ClientMessage,
    },
    ServerMessage {
        from_user: UserName,
        message: ServerMessage,
    },
    NewUser(UserName, mpsc::Sender<ProcessMessage>),
    Response(ProcessResponse),
    Complete,
}

#[derive(Debug, Clone)]
pub enum ProcessResponse {
    Complete,
    Error,
}
