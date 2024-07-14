use crate::Frame;

pub type Result<T> = std::result::Result<T, Error>;
//pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, derive_more::From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
    ConnectionDropped,
    DropConnectionFailed,
    #[from]
    Bincode(bincode::Error),
    #[from]
    BroadcastFailed(tokio::sync::broadcast::error::SendError<Frame>),
    #[from]
    BroadcastFailed1(tokio::sync::broadcast::error::SendError<(String, Frame)>),
    #[from]
    MpscSendFailed(tokio::sync::mpsc::error::SendError<Frame>),

    InvalidHandshake,

    //Remove
    ImplementFrame,
}

//Error boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self)
    }
}

impl std::error::Error for Error {}
