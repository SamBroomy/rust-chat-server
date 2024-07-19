use crate::common::UserName;
use crate::connection::FrameType;

use bincode::{Decode, Encode};
use std::fmt::Display;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Handshake(pub UserName);

impl Display for Handshake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handshake: {}", self.0)
    }
}

impl FrameType for Handshake {}
