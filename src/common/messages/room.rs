use crate::common::{RoomName, UserName};

#[derive(Debug)]
pub struct RoomMessage {
    pub from_user: UserName,
    pub room_name: RoomName,
    pub message: RoomInternal,
}

#[derive(Debug)]
pub enum RoomInternal {
    NewRoom,
    JoinRoom,
    LeaveRoom,
    ListRooms,
    ListUsers,
    RoomMessage(String),
}
