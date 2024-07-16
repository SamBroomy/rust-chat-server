//! After looking at the tokio implementation I think the way they handle reading frames is a bit more complex than it needs to be for this project.
use crate::{FrameType, Room, User};

use bincode::{Decode, Encode};

use crossterm::style::Stylize;

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone, Decode, Encode)]
pub enum ClientMessage {
    Handshake(User),
    ChatMessage {
        content: String,
    },
    Ping(u64),
    Join {
        room: String,
    },
    Create {
        room: String,
        description: Option<String>,
    },
    Leave,
    ListRooms,
    ListUsers,
    Disconnect,
    PrivateMessage {
        to_user: User,
        content: String,
    },
}

impl FrameType for ClientMessage {}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientMessage::Handshake(user) => write!(f, "User {} connected", user),
            ClientMessage::ChatMessage { content } => write!(f, "{}", content),
            ClientMessage::Ping(i) => write!(f, "Ping: {:}", i),
            ClientMessage::Join { room } => write!(f, "Joining room: {}", room),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ServerMessage {
    ServerMessage { content: String },
    ChatMessage { from: User, content: String },
    PrivateMessage { from: User, content: String },
    Error { message: String },
    Pong(u64),
    RoomList { rooms: Vec<String> },
    UserList { users: Vec<String> },
    RoomJoined { room: Room, user: User },
    RoomCreated { room: String },
    RoomLeft { room: String },
    UserJoined { user: String },
}

impl FrameType for ServerMessage {}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerMessage::ServerMessage { content } => write!(f, "Server: {}", content),
            ServerMessage::ChatMessage { from, content } => {
                write!(f, "{:<10}: {}", from.username().yellow(), content)
            }
            ServerMessage::PrivateMessage { from, content } => {
                write!(
                    f,
                    "{} {:<10}: {}",
                    "[PrivateMessage]".dark_magenta(),
                    from.to_string().magenta(),
                    content
                )
            }
            ServerMessage::Error { message } => write!(f, "Error: {}", message),
            ServerMessage::Pong(i) => write!(f, "Pong: {:}", i),
            ServerMessage::RoomList { rooms } => {
                write!(f, "Rooms: {}", rooms.join(", "))
            }
            ServerMessage::UserList { users } => {
                write!(f, "Users: {}", users.join(", "))
            }
            ServerMessage::RoomJoined { room, user } => {
                write!(f, "User {} joined room: {}", user, room)
            }
            ServerMessage::RoomCreated { room } => write!(f, "Created room: {}", room),
            ServerMessage::RoomLeft { room } => write!(f, "Left room: {}", room),
            ServerMessage::UserJoined { user } => write!(f, "User joined: {}", user),
        }
    }
}

impl ServerMessage {
    pub fn get_user(&self) -> Option<User> {
        match self {
            ServerMessage::ChatMessage { from, .. } => Some(from.clone()),
            ServerMessage::PrivateMessage { from, .. } => Some(from.clone()),
            ServerMessage::RoomJoined { user, .. } => Some(user.clone()),
            _ => None,
        }
    }
}
