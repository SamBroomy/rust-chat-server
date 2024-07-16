use crate::{FrameType, Room, User};
use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone, Decode, Encode)]
pub enum ServerMessage {
    ServerMessage(String),
    ChatMessage { from: User, content: String },
    PrivateMessage { from: User, content: String },
    Error(String),
    Pong(u16),
    RoomList { rooms: Vec<Room> },
    UserList { users: Vec<User> },
    RoomJoined { room: Room, user: User },
    RoomCreated(Room),
    RoomLeft(Room),
    UserJoined(User),
}

// Builder methods
impl ServerMessage {
    #[allow(clippy::self_named_constructors)]
    pub fn server_message(content: impl Into<String>) -> Self {
        Self::ServerMessage(content.into())
    }

    pub fn chat_message(from: impl Into<User>, content: impl Into<String>) -> Self {
        Self::ChatMessage {
            from: from.into(),
            content: content.into(),
        }
    }

    pub fn private_message(from: impl Into<User>, content: impl Into<String>) -> Self {
        Self::PrivateMessage {
            from: from.into(),
            content: content.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    pub fn pong(i: u16) -> Self {
        Self::Pong(i)
    }

    pub fn room_list(rooms: Vec<Room>) -> Self {
        Self::RoomList { rooms }
    }

    pub fn user_list(users: Vec<User>) -> Self {
        Self::UserList { users }
    }

    pub fn room_joined(room: impl Into<Room>, user: impl Into<User>) -> Self {
        Self::RoomJoined {
            room: room.into(),
            user: user.into(),
        }
    }
}

impl FrameType for ServerMessage {}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerMessage::ServerMessage(content) => {
                write!(
                    f,
                    "{} {}",
                    "Server:".underline_dark_red(),
                    content.as_str().red()
                )
            }
            ServerMessage::ChatMessage { from, content } => {
                write!(f, "{:<10}: {}", from.username().yellow(), content)
            }
            ServerMessage::PrivateMessage { from, content } => {
                write!(
                    f,
                    "{} {:<10}: {}",
                    "[PrivateMessage]".dark_magenta(),
                    from.to_string().magenta(),
                    content.as_str().grey()
                )
            }
            ServerMessage::Error(message) => {
                write!(
                    f,
                    "{} {}",
                    "Error:".bold().on_dark_red(),
                    message.as_str().red()
                )
            }
            ServerMessage::Pong(i) => write!(f, "{}", format!("Pong: {:}", i).yellow()),
            ServerMessage::RoomList { rooms } => {
                write!(
                    f,
                    "{}, {}",
                    "[Rooms]".on_cyan(),
                    rooms
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ServerMessage::UserList { users } => {
                write!(
                    f,
                    "{} {}",
                    "[Users]".on_yellow(),
                    users
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ServerMessage::RoomJoined { room, user } => {
                write!(
                    f,
                    "User {} joined room: {}",
                    user.to_string().on_yellow(),
                    room.to_string().on_cyan()
                )
            }
            ServerMessage::RoomCreated(room) => {
                write!(f, "Created room: {}", room.to_string().cyan())
            }
            ServerMessage::RoomLeft(room) => write!(f, "Left room: {}", room.to_string().cyan()),
            ServerMessage::UserJoined(user) => {
                write!(f, "User joined: {}", user.to_string().on_yellow())
            }
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
