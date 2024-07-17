mod error;
mod frame;

pub use error::ConnectionError;
use error::Result;
pub use frame::FrameType;

use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::info;

pub type Reader<'a> = BufReader<ReadHalf<'a>>;
pub type Writer<'a> = BufWriter<WriteHalf<'a>>;
pub type OwnedReader = BufReader<OwnedReadHalf>;
pub type OwnedWriter = BufWriter<OwnedWriteHalf>;
#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub async fn init(addr: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(ConnectionError::UnableToConnectToServer)?;
        info!("Connected to server {}", stream.peer_addr()?);

        Ok(Self { stream })
    }

    pub fn from_stream(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Convenience method to read a frame from the stream
    pub async fn read_frame<F: FrameType>(&mut self) -> Result<F> {
        F::read_frame_from(&mut self.stream.split().0).await
    }

    /// Convenience method to write a frame to the stream
    pub async fn write_frame<F: FrameType>(&mut self, frame: &F) -> Result<()> {
        frame.write_frame_to(&mut self.stream.split().1).await
    }

    pub fn split_into(self) -> (OwnedReader, OwnedWriter) {
        let (reader, writer) = self.stream.into_split();

        (BufReader::new(reader), BufWriter::new(writer))
    }

    pub fn split(&mut self) -> (Reader<'_>, Writer<'_>) {
        let (reader, writer) = self.stream.split();
        (BufReader::new(reader), BufWriter::new(writer))
    }

    pub fn split_owned(stream: TcpStream) -> (OwnedReader, OwnedWriter) {
        let (reader, writer) = stream.into_split();
        (BufReader::new(reader), BufWriter::new(writer))
    }

    pub async fn close(&mut self) {
        let _ = self.stream.shutdown().await;
    }
}
