//! After looking at the tokio implementation I think the way they handle reading frames is a bit more complex than it needs to be for this project.
use crate::{Error, Result};

use async_trait::async_trait;
use bincode;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tracing::{debug, error, instrument};

#[async_trait]
pub trait FrameType: Serialize + for<'a> Deserialize<'a> + Debug {
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
pub enum Frame {
    // Server communication frames
    Handshake {
        username: String,
    },
    ServerMessage {
        content: String,
    },

    // Chat message frame
    ChatMessage {
        username: String,
        content: String,
    },

    // Error frame
    Error {
        message: String,
    },

    // Ping frame
    Ping(u64),
    Pong(u64),

    // Server-related commands
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

    // // Chat-related messages
    // ChatMessage {
    //     room: String,
    //     content: String,
    // },

    // Server responses
    ServerResponse {
        content: String,
    },
    RoomList {
        rooms: Vec<String>,
    },
    UserList {
        users: Vec<String>,
    },

    // Connection-related
    Connect {
        username: String,
    },
    Disconnect,
}

impl FrameType for Frame {}

impl ToString for Frame {
    fn to_string(&self) -> String {
        match self {
            Frame::Handshake { username } => format!("{} connected", username),
            Frame::ServerMessage { content } => content.clone(),
            Frame::ChatMessage { username, content } => format!("{}: {}", username, content),
            Frame::Error { message } => format!("Error: {}", message),
            Frame::Ping(i) => format!("Ping: {:}", i),
            Frame::Pong(i) => format!("Pong: {:}", i),
            _ => todo!(),
        }
    }
}
