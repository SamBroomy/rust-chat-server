use crate::{FrameType, Result};

use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    writer: BufWriter<OwnedWriteHalf>,
    reader: BufReader<OwnedReadHalf>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self {
            writer: BufWriter::new(writer),
            reader: BufReader::new(reader),
        }
    }

    pub fn from_parts(reader: OwnedReadHalf, writer: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::new(writer),
            reader: BufReader::new(reader),
        }
    }

    /// Convenience method to read a frame from the stream
    pub async fn read_frame<F: FrameType>(&mut self) -> Result<F> {
        F::read_from(&mut self.reader).await
    }

    /// Convenience method to write a frame to the stream
    pub async fn write_frame<F: FrameType>(&mut self, frame: &F) -> Result<()> {
        frame.write_to(&mut self.writer).await
    }

    pub fn get_writer(&mut self) -> &mut BufWriter<OwnedWriteHalf> {
        &mut self.writer
    }

    pub fn get_reader(&mut self) -> &mut BufReader<OwnedReadHalf> {
        &mut self.reader
    }

    pub fn split_into(self) -> (BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>) {
        (self.reader, self.writer)
    }

    pub fn split(stream: &mut TcpStream) -> (BufReader<ReadHalf>, BufWriter<WriteHalf>) {
        let (reader, writer) = stream.split();
        (BufReader::new(reader), BufWriter::new(writer))
    }
}
