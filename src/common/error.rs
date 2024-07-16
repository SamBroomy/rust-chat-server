use crate::Room;

pub type Result<T> = std::result::Result<T, CommonError>;

#[derive(Debug)]
pub enum CommonError {
    RoomExists(Room),
    RoomNotFound(Room),
}

//Error boilerplate
impl core::fmt::Display for CommonError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CommonError {}
