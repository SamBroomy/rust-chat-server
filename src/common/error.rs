pub type Result<T> = std::result::Result<T, CommonError>;

use super::{RoomName, UserName};

#[derive(Debug)]
pub enum CommonError {
    UserExists(UserName),
    UserNotExists(UserName),
    RoomExists(RoomName),
    RoomNotFound(RoomName),
}

//Error boilerplate
impl core::fmt::Display for CommonError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CommonError {}
