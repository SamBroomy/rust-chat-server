use crate::{common::RoomName, connection::FrameType};

use crate::common::UserName;

use bincode::{Decode, Encode};
use std::fmt::{self, Debug, Display, Formatter};

/// Messages sent by the client to the server
#[derive(Debug, Clone, Decode, Encode)]
pub enum ClientMessage {
    GlobalChatMessage(String),
    PrivateMessage { to_user: UserName, content: String },
    Ping(u16),
    ListUsers,
    Disconnect,
    CreateRoom(RoomName),
    JoinRoom(RoomName),
    LeaveRoom(RoomName),
    ListRooms,
    ListRoomUsers(RoomName),
    RoomMessage { room: RoomName, content: String },
}

impl FrameType for ClientMessage {}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientMessage::GlobalChatMessage(content) => write!(f, "{}", content),
            ClientMessage::Ping(i) => write!(f, "Ping: {}", i),
            ClientMessage::ListUsers => write!(f, "Listing users"),
            ClientMessage::Disconnect => write!(f, "Disconnecting"),
            ClientMessage::PrivateMessage { to_user, content } => {
                write!(f, "Private message to {}: {}", to_user, content)
            }
            ClientMessage::CreateRoom(room) => write!(f, "Creating room: {}", room),
            ClientMessage::JoinRoom(room) => write!(f, "Joining room: {}", room),
            ClientMessage::LeaveRoom(room) => write!(f, "Leaving room: {}", room),
            ClientMessage::ListRoomUsers(room) => {
                write!(f, "Listing users in room: {:?}", room)
            }
            ClientMessage::ListRooms => write!(f, "Listing rooms"),
            ClientMessage::RoomMessage { room, content } => {
                write!(f, "Room message to {}: {}", room, content)
            }
        }
    }
}
