//! After looking at the tokio implementation I think the way they handle reading frames is a bit more complex than it needs to be for this project.
use crate::{Error, Result};

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tracing::{debug, error, instrument};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct User {
    username: String,
    id: u32,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            id: rand::random(),
        }
    }
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.username, self.id)
    }
}

impl From<String> for User {
    fn from(username: String) -> Self {
        Self::new(username)
    }
}

impl From<&str> for User {
    fn from(username: &str) -> Self {
        Self::new(username.to_string())
    }
}

#[async_trait]
pub trait FrameType: Serialize + for<'a> Deserialize<'a> + Debug + Send + Sync + Display {
    #[instrument(skip(self), level = "trace", ret, err)]
    fn serialize_frame(&self) -> Result<Bytes> {
        Ok(bincode::serialize(self)?.into())
    }

    #[instrument(skip(data), level = "trace", ret, err)]
    fn deserialize_frame(data: &impl AsRef<[u8]>) -> Result<Self> {
        bincode::deserialize(data.as_ref())
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, e).into())
    }

    async fn read_from<R: AsyncRead + Unpin + Send>(reader: &mut BufReader<R>) -> Result<Self> {
        let mut buffer = BytesMut::new();
        if 0 == reader.read_buf(&mut buffer).await? {
            error!("Connection dropped");
            return Err(Error::ConnectionDropped);
        }
        debug!("Frame received from stream");
        Self::deserialize_frame(&buffer)
    }

    async fn write_to<W: AsyncWriteExt + Unpin + Send>(
        &self,
        writer: &mut BufWriter<W>,
    ) -> Result<()> {
        let data = bincode::serialize(self)?;
        writer.write_all(&data).await?;
        writer.flush().await?;
        debug!("Frame written to stream");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientFrame {
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
    Help,
    Disconnect,
}

impl FrameType for ClientFrame {}

impl Display for ClientFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientFrame::Handshake(user) => write!(f, "User {} connected", user),
            ClientFrame::ChatMessage { content } => write!(f, "{}", content),
            ClientFrame::Ping(i) => write!(f, "Ping: {:}", i),
            ClientFrame::Join { room } => write!(f, "Joining room: {}", room),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerFrame {
    ServerMessage { content: String },
    ChatMessage { user: User, content: String },
    Error { message: String },
    Pong(u64),
    RoomList { rooms: Vec<String> },
    UserList { users: Vec<String> },
}

impl FrameType for ServerFrame {}

impl Display for ServerFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerFrame::ServerMessage { content } => write!(f, "Server: {}", content),
            ServerFrame::ChatMessage { user, content } => {
                write!(f, "{:<10}: {}", user.username().yellow(), content)
            }
            ServerFrame::Error { message } => write!(f, "Error: {}", message),
            ServerFrame::Pong(i) => write!(f, "Pong: {:}", i),
            ServerFrame::RoomList { rooms } => {
                write!(f, "Rooms: {}", rooms.join(", "))
            }
            ServerFrame::UserList { users } => {
                write!(f, "Users: {}", users.join(", "))
            }
        }
    }
}
