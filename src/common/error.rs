pub type Result<T> = std::result::Result<T, CommonError>;

use super::RoomName;

#[derive(Debug)]
pub enum CommonError {
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
