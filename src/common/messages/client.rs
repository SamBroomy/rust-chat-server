use crate::{FrameType, RoomName, UserName};
use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use std::fmt::{self, Debug, Display, Formatter};

/// Messages sent by the client to the server
#[derive(Debug, Clone, Decode, Encode)]
pub enum ClientMessage {
    Handshake(UserName),
    GlobalChatMessage(String),
    RoomMessage { room: RoomName, content: String },
    PrivateMessage { to_user: UserName, content: String },
    Ping(u16),
    Join(RoomName),
    CreateRoom(RoomName),
    Leave,
    ListRooms,
    ListUsers,
    Disconnect,
}

// Builder methods
impl ClientMessage {
    pub fn handshake(user: impl Into<UserName>) -> Self {
        Self::Handshake(user.into())
    }

    pub fn global_chat_message(content: impl Into<String>) -> Self {
        Self::GlobalChatMessage(content.into())
    }

    pub fn private_message(to_user: impl Into<UserName>, content: impl Into<String>) -> Self {
        Self::PrivateMessage {
            to_user: to_user.into(),
            content: content.into(),
        }
    }

    pub fn ping(i: u16) -> Self {
        Self::Ping(i)
    }

    pub fn join(room: impl Into<RoomName>) -> Self {
        Self::Join(room.into())
    }

    pub fn create_room(room: impl Into<RoomName>) -> Self {
        Self::CreateRoom(room.into())
    }
}

impl FrameType for ClientMessage {}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientMessage::Handshake(user) => {
                write!(
                    f,
                    "{}",
                    format!("User {} connected!", user.to_string().bold()).green()
                )
            }
            ClientMessage::GlobalChatMessage(content) => write!(f, "{}", content),
            ClientMessage::RoomMessage { room, content } => {
                write!(f, "{}: {}", room.to_string().bold(), content)
            }
            ClientMessage::Ping(i) => write!(f, "Ping: {}", i),
            ClientMessage::Join(room) => write!(f, "Joining room: {}", room),
            ClientMessage::CreateRoom(room) => write!(f, "Creating room: {}", room),
            ClientMessage::Leave => write!(f, "Leaving room"),
            ClientMessage::ListRooms => write!(f, "Listing rooms"),
            ClientMessage::ListUsers => write!(f, "Listing users"),
            ClientMessage::Disconnect => write!(f, "Disconnecting"),
            ClientMessage::PrivateMessage { to_user, content } => {
                write!(f, "Private message to {}: {}", to_user, content)
            }
        }
    }
}
