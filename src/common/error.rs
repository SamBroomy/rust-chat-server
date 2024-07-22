pub type Result<T> = std::result::Result<T, CommonError>;

use super::{
    messages::{ServerMessage, UserMessage},
    RoomName, User, UserName,
};

#[derive(Debug, derive_more::From)]
pub enum CommonError {
    UserExists(UserName),
    UserInRoom(User),
    UserNotExists(UserName),
    UserNotInRoom(User),
    RoomExists(RoomName),
    NoUsersInRoom,
    RoomMessageNotSent,
    RoomNotFound(RoomName),
    #[from]
    SendUserProcess(tokio::sync::mpsc::error::SendError<UserMessage>),
    #[from]
    SendUserProcessBroadcast(tokio::sync::mpsc::error::SendError<ServerMessage>),
}

//Error boilerplate
impl core::fmt::Display for CommonError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CommonError {}
