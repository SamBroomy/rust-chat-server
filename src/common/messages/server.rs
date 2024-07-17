use crate::{FrameType, RoomName, UserName};
use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone, Decode, Encode)]
pub enum ServerMessage {
    ServerMessage(String),
    ChatMessage {
        from: UserName,
        content: String,
    },
    RoomMessage {
        room: RoomName,
        from: UserName,
        content: String,
    },
    PrivateMessage {
        from: UserName,
        content: String,
    },
    Error(String),
    Pong(u16),
    RoomList {
        rooms: Vec<RoomName>,
    },
    UserList {
        users: Vec<UserName>,
    },
    RoomJoined {
        room: RoomName,
        user: UserName,
    },
    RoomCreated(RoomName),
    RoomLeft(RoomName),
    UserJoined(UserName),
}

// Builder methods
impl ServerMessage {
    #[allow(clippy::self_named_constructors)]
    pub fn server_message(content: impl Into<String>) -> Self {
        Self::ServerMessage(content.into())
    }

    pub fn chat_message(from: impl Into<UserName>, content: impl Into<String>) -> Self {
        Self::ChatMessage {
            from: from.into(),
            content: content.into(),
        }
    }

    pub fn private_message(from: impl Into<UserName>, content: impl Into<String>) -> Self {
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

    pub fn room_list(rooms: Vec<RoomName>) -> Self {
        Self::RoomList { rooms }
    }

    pub fn user_list(users: Vec<UserName>) -> Self {
        Self::UserList { users }
    }

    pub fn room_joined(room: impl Into<RoomName>, user: impl Into<UserName>) -> Self {
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
            ServerMessage::RoomMessage {
                room,
                from,
                content,
            } => {
                write!(
                    f,
                    "{} {:<10}: {}",
                    format!("[{}]", room).to_string().cyan(),
                    from.to_string().yellow(),
                    content
                )
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
                let rooms = rooms.iter().map(ToString::to_string).collect::<Vec<_>>();
                if rooms.is_empty() {
                    write!(f, "{}", "[No Rooms created]".cyan())
                } else {
                    write!(f, "{}, {}", "[Rooms]".cyan(), rooms.join(", ").cyan())
                }
            }
            ServerMessage::UserList { users } => {
                let users = users.iter().map(ToString::to_string).collect::<Vec<_>>();
                if users.is_empty() {
                    write!(f, "{}", "[No Users]".yellow())
                } else {
                    write!(f, "{} {}", "[Users]".yellow(), users.join(", "))
                }
            }
            ServerMessage::RoomJoined { room, user } => {
                write!(
                    f,
                    "User {} joined room: {}",
                    user.to_string().yellow(),
                    room.to_string().cyan()
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
    pub fn get_user(&self) -> Option<UserName> {
        match self {
            ServerMessage::ChatMessage { from, .. } => Some(from.clone()),
            ServerMessage::PrivateMessage { from, .. } => Some(from.clone()),
            ServerMessage::RoomJoined { user, .. } => Some(user.clone()),
            _ => None,
        }
    }
}
