use crate::common::messages::{ProcessMessage, ServerMessage, UserMessage};
use crate::common::UserName;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, derive_more::From)]
pub enum ServerError {
    #[from]
    Connection(crate::connection::ConnectionError),
    #[from]
    Io(std::io::Error),
    #[from]
    ServerBroadcastFailed(tokio::sync::broadcast::error::SendError<ServerMessage>),
    #[from]
    ClientBroadcastFailed(tokio::sync::mpsc::error::SendError<ProcessMessage>),
    #[from]
    OutputBroadcastFailed(tokio::sync::mpsc::error::SendError<ServerMessage>),
    #[from]
    UserBroadcastFailed(tokio::sync::mpsc::error::SendError<UserMessage>),
    #[from]
    Common(crate::common::CommonError),
    InvalidHandshake,
    HandshakeTimeout,
    UserNotFound(UserName),
}

//Error boilerplate
impl core::fmt::Display for ServerError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ServerError {}
