use super::{ConnectionError, Result};

use async_trait::async_trait;
use bincode::{config, Decode, Encode};
use bytes::BytesMut;
use std::fmt::{Debug, Display};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, warn};

#[async_trait]
pub trait FrameType: Debug + Display + Encode + Decode + Send + Sync {
    async fn write_frame_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWriteExt + Unpin + Send,
    {
        // First encode the data to a vec.
        let data = bincode::encode_to_vec(self, config::standard())?;
        // Write the length of the frame to the stream, this is a u32 so 4 bytes. These 4 bytes will be used to read the frame from the stream.
        writer.write_u32_le(data.len() as u32).await?;
        // Then, write the actual frame
        writer.write_all(&data).await?;
        // Flush the writer to ensure that the data is sent to the stream.
        writer.flush().await?;
        Ok(())
    }

    /// This function is cancel safe as it is used in a select!.
    async fn read_frame_from<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncReadExt + Unpin + Send,
    {
        // Helper to convert io errors to ConnectionError
        let convert_err = |e: std::io::Error| match e.kind() {
            std::io::ErrorKind::UnexpectedEof => {
                warn!("Connection closed!");
                ConnectionError::ConnectionClosed
            }
            _ => {
                error!("Connection dropped while reading frame size");
                ConnectionError::ConnectionDropped
            }
        };
        // First, read the length of the frame from the stream. This needs to be 4 bytes long as our frame size definition is u32
        let mut size_buf = [0u8; 4];
        reader
            .read_exact(&mut size_buf)
            .await
            .map_err(convert_err)?;
        // Convert the buffer to a u32
        let size = u32::from_le_bytes(size_buf) as usize;
        // Define a buffer with the exact size of the frame, as specified in the first 4 bytes.
        let mut buf = BytesMut::with_capacity(size);
        reader.read_buf(&mut buf).await.map_err(convert_err)?;
        // Maybe pointless to freeze, but ensures that the buffer is not modified and we have the underlying bytes.
        let data = buf.freeze();
        // Check if the buffer size is the same as the size of the frame
        if size != data.len() {
            error!("Invalid frame size: {} != {}", size, data.len());
            return Err(ConnectionError::InvalidFrameSize);
        }
        // Decode the frame from the buffer
        let (frame, decode_size) = bincode::decode_from_slice(&data, config::standard())?;
        assert_eq!(size, decode_size);
        Ok(frame)
    }
}
