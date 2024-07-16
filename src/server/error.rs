use crate::{ServerMessage, User};

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, derive_more::From)]
pub enum ServerError {
    #[from]
    Connection(crate::connection::ConnectionError),
    #[from]
    Io(std::io::Error),
    #[from]
    ServerBroadcastFailed(tokio::sync::broadcast::error::SendError<(User, ServerMessage)>),
    #[from]
    ClientBroadcastFailed(tokio::sync::mpsc::error::SendError<(User, ServerMessage)>),

    InvalidHandshake,
    UserNotFound(User),
}

//Error boilerplate
impl core::fmt::Display for ServerError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ServerError {}
