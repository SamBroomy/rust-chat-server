use crate::common::UserName;
use crate::connection::FrameType;

use bincode::{Decode, Encode};
use crossterm::style::Stylize;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone, Decode, Encode)]
pub struct ServerMessage {
    pub from_user: UserName,
    pub content: ServerInternal,
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.from_user, self.content)
    }
}
#[derive(Debug, Clone, Decode, Encode)]
pub enum ServerInternal {
    ServerMessage(String),
    GlobalChatMessage {
        from_user: UserName,
        content: String,
    },
    ChatMessage(String),
    PrivateMessage {
        from_user: UserName,
        content: String,
    },
    UserJoined(UserName),
    UserList {
        users: Vec<UserName>,
    },
    Error(String),
    Pong(u16),
    // RoomMessage {
    //     room: RoomName,
    //     from: UserName,
    //     content: String,
    // },
    // RoomList {
    //     rooms: Vec<RoomName>,
    // },
    // RoomJoined {
    //     room: RoomName,
    //     user: UserName,
    // },
    // RoomCreated(RoomName),
    // RoomLeft(RoomName),
}

impl FrameType for ServerInternal {}

impl Display for ServerInternal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerInternal::ServerMessage(content) => {
                write!(
                    f,
                    "{} {}",
                    "Server:".underline_dark_red(),
                    content.as_str().red()
                )
            }
            ServerInternal::ChatMessage(content) => {
                write!(f, "{}", content)
            }
            ServerInternal::GlobalChatMessage { from_user, content } => {
                write!(
                    f,
                    "{} {:<10}: {}",
                    "[Global]".dark_green(),
                    from_user.to_string().green(),
                    content.as_str().green()
                )
            }
            // ServerMessage::RoomMessage {
            //     room,
            //     from,
            //     content,
            // } => {
            //     write!(
            //         f,
            //         "{} {:<10}: {}",
            //         format!("[{}]", room).to_string().cyan(),
            //         from.to_string().yellow(),
            //         content
            //     )
            // }
            ServerInternal::PrivateMessage { from_user, content } => {
                write!(
                    f,
                    "{} {:<10}: {}",
                    "[PrivateMessage]".dark_magenta(),
                    from_user.to_string().magenta(),
                    content.as_str().grey()
                )
            }
            ServerInternal::Error(message) => {
                write!(
                    f,
                    "{} {}",
                    "Error:".bold().on_dark_red(),
                    message.as_str().red()
                )
            }
            ServerInternal::Pong(i) => write!(f, "{}", format!("Pong: {:}", i).yellow()),
            // ServerMessage::RoomList { rooms } => {
            //     let rooms = rooms.iter().map(ToString::to_string).collect::<Vec<_>>();
            //     if rooms.is_empty() {
            //         write!(f, "{}", "[No Rooms created]".cyan())
            //     } else {
            //         write!(f, "{}, {}", "[Rooms]".cyan(), rooms.join(", ").cyan())
            //     }
            // }
            ServerInternal::UserList { users } => {
                let users = users.iter().map(ToString::to_string).collect::<Vec<_>>();
                if users.is_empty() {
                    write!(f, "{}", "[No Users]".yellow())
                } else {
                    write!(f, "{} {}", "[Users]".yellow(), users.join(", "))
                }
            }
            // ServerMessage::RoomJoined { room, user } => {
            //     write!(
            //         f,
            //         "User {} joined room: {}",
            //         user.to_string().yellow(),
            //         room.to_string().cyan()
            //     )
            // }
            // ServerMessage::RoomCreated(room) => {
            //     write!(f, "Created room: {}", room.to_string().cyan())
            // }
            // ServerMessage::RoomLeft(room) => write!(f, "Left room: {}", room.to_string().cyan()),
            ServerInternal::UserJoined(user) => {
                write!(f, "User joined: {}", user.to_string().on_yellow())
            }
        }
    }
}
