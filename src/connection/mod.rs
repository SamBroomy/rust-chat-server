mod error;
mod frame;

pub use error::ConnectionError;
use error::Result;
pub use frame::FrameType;

use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

pub type Reader = BufReader<OwnedReadHalf>;
pub type Writer = BufWriter<OwnedWriteHalf>;

#[derive(Debug)]
pub struct Connection {
    reader: Reader,
    writer: Writer,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
        }
    }

    pub fn from_parts(reader: OwnedReadHalf, writer: OwnedWriteHalf) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
        }
    }

    /// Convenience method to read a frame from the stream
    pub async fn read_frame<F: FrameType>(&mut self) -> Result<F> {
        F::read_frame_from(&mut self.reader).await
    }

    /// Convenience method to write a frame to the stream
    pub async fn write_frame<F: FrameType>(&mut self, frame: &F) -> Result<()> {
        frame.write_frame_to(&mut self.writer).await
    }

    pub fn get_writer(&mut self) -> &mut Writer {
        &mut self.writer
    }

    pub fn get_reader(&mut self) -> &mut Reader {
        &mut self.reader
    }

    pub fn split_into(self) -> (Reader, Writer) {
        (self.reader, self.writer)
    }

    pub fn split(stream: &mut TcpStream) -> (BufReader<ReadHalf>, BufWriter<WriteHalf>) {
        let (reader, writer) = stream.split();
        (BufReader::new(reader), BufWriter::new(writer))
    }

    pub fn split_owned(stream: TcpStream) -> (Reader, Writer) {
        let (reader, writer) = stream.into_split();
        (BufReader::new(reader), BufWriter::new(writer))
    }
}
