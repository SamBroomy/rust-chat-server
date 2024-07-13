//! After looking at the tokio implementation I think the way they handle reading frames is a bit more complex than it needs to be for this project.
use crate::Result;

use bincode;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frame {
    Text { sender: String, content: String },
    Join { username: String },
    Leave { username: String },
    Null,
}

impl Frame {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize frame")
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
    }

    // pub fn write_to<W: AsyncWriteExt>(&self, writer: &mut W) -> io::Result<()> {
    //     let data = self.serialize();
    //     writer.write_all(&(data.len() as u32).to_be_bytes())?;
    //     writer.write_all(&data)
    // }

    // pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
    //     let mut len_bytes = [0u8; 4];
    //     reader.read_exact(&mut len_bytes)?;
    //     let len = u32::from_be_bytes(len_bytes) as usize;

    //     let mut data = vec![0u8; len];
    //     reader.read_exact(&mut data)?;

    //     Self::deserialize(&data)
    // }
}
