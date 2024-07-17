use super::{CommonError, Result};
use crate::{ServerMessage, UserName};

use bincode::{Decode, Encode};
use std::collections::HashMap;
use std::fmt::Display;
use tokio::sync::broadcast;
use tracing::error;

#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq, Hash)]
pub struct RoomName {
    name: String,
}

impl Display for RoomName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for RoomName {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl From<&str> for RoomName {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RoomManager {
    rooms: HashMap<RoomName, broadcast::Sender<(UserName, ServerMessage)>>,
}

impl RoomManager {
    pub fn create_room(&mut self, room: impl Into<RoomName>) -> Result<RoomName> {
        let room = room.into();
        if self.rooms.contains_key(&room) {
            // Update key if room exists?
            return Err(CommonError::RoomExists(room));
        }
        let (tx, _) = broadcast::channel(100);

        self.rooms.insert(room.clone(), tx);
        Ok(room)
    }

    pub fn update_room(
        &mut self,
        old_room: impl Into<RoomName>,
        new_room: impl Into<RoomName>,
    ) -> Result<()> {
        let old_room = old_room.into();
        let new_room = new_room.into();

        if self.rooms.contains_key(&new_room) {
            return Err(CommonError::RoomExists(new_room));
        }

        // Temporarily remove the entry to get ownership of the value.
        if let Some(tx) = self.rooms.remove(&old_room) {
            // Insert the value back with the new key.
            self.rooms.insert(new_room, tx);
            Ok(())
        } else {
            Err(CommonError::RoomNotFound(old_room))
        }
    }

    pub fn join_room(
        &self,
        room: &RoomName,
        user: &UserName,
    ) -> Result<broadcast::Receiver<(UserName, ServerMessage)>> {
        self.rooms
            .get(room)
            .map(|tx| {
                if let Err(e) = tx.send((
                    user.clone(),
                    ServerMessage::RoomJoined {
                        room: room.clone(),
                        user: user.clone(),
                    },
                )) {
                    error!("Failed to send room joined message: {}", e);
                }
                tx.subscribe()
            })
            .ok_or(CommonError::RoomNotFound(room.clone()))
    }

    pub fn leave_room(&mut self, room: &RoomName, user: &UserName) -> Result<()> {
        if let Some(tx) = self.rooms.get(room) {
            if let Err(e) = tx.send((user.clone(), ServerMessage::RoomLeft(room.clone()))) {
                error!("Failed to send room left message: {}", e);
            }
            Ok(())
        } else {
            Err(CommonError::RoomNotFound(room.clone()))
        }
    }

    pub fn list_rooms(&self) -> Vec<RoomName> {
        self.rooms.keys().cloned().collect()
    }

    pub fn send_message(
        &self,
        room: &RoomName,
        user: &UserName,
        message: ServerMessage,
    ) -> Result<()> {
        if let Some(tx) = self.rooms.get(room) {
            if let Err(e) = tx.send((user.clone(), message)) {
                error!("Failed to send message: {}", e);
            }
            Ok(())
        } else {
            Err(CommonError::RoomNotFound(room.clone()))
        }
    }
}
