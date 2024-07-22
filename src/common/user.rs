use super::messages::ServerMessage;
use super::{CommonError, Result};

use bincode::{Decode, Encode};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
};
use tokio::sync::mpsc;
use tracing::error;

#[derive(Debug, Clone, Encode, Decode, PartialEq, Hash, Eq)]
pub struct UserName {
    user_name: String,
}

impl UserName {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            user_name: username.into(),
        }
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_name)
    }
}

impl PartialEq<&str> for UserName {
    fn eq(&self, other: &&str) -> bool {
        self.user_name == *other
    }
}

impl PartialEq<String> for UserName {
    fn eq(&self, other: &String) -> bool {
        self.user_name == *other
    }
}

impl From<String> for UserName {
    fn from(username: String) -> Self {
        Self::new(username)
    }
}

impl From<&str> for UserName {
    fn from(username: &str) -> Self {
        Self::new(username.to_string())
    }
}

// trait RoomStatus {
//     fn is_in_room() -> bool;
// }

// struct NoRoom;

// struct InRoom {
//     room_name: RoomName,
// }

// impl RoomStatus for NoRoom {
//     fn is_in_room() -> bool {
//         false
//     }
// }

// impl RoomStatus for InRoom {
//     fn is_in_room() -> bool {
//         true
//     }
// }

#[derive(Debug, Clone)]
pub struct User {
    user_name: UserName,
    user_tx: mpsc::Sender<ServerMessage>,
}

impl Eq for User {}

impl PartialEq<User> for User {
    fn eq(&self, other: &User) -> bool {
        self.user_name() == other.user_name()
    }
}

impl Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_name().hash(state);
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_name)
    }
}

impl User {
    pub fn new(user_name: impl Into<UserName>) -> (Self, mpsc::Receiver<ServerMessage>) {
        let (user_tx, user_rx) = mpsc::channel(16);
        (
            Self {
                user_name: user_name.into(),
                user_tx,
            },
            user_rx,
        )
    }

    pub fn user_name(&self) -> &UserName {
        &self.user_name
    }

    pub fn user_tx(&self) -> mpsc::Sender<ServerMessage> {
        self.user_tx.clone()
    }
}

#[derive(Default)]
pub struct UserManager {
    users: HashMap<UserName, User>,
}

impl UserManager {
    pub fn add_new_user(
        &mut self,
        user_name: impl Into<UserName>,
    ) -> Result<mpsc::Receiver<ServerMessage>> {
        let (user, user_rx) = User::new(user_name);
        self.add_user(user)?;
        Ok(user_rx)
    }

    /// Returns none if new value, returns Some if user information updated.
    pub fn add_user(&mut self, user: User) -> Result<()> {
        if self.users.contains_key(user.user_name()) {
            error!("User Already exists");
            return Err(CommonError::UserExists(user.user_name().clone()));
        }
        self.users.insert(user.user_name().clone(), user);
        Ok(())
    }

    pub fn remove_user(&mut self, user_name: &UserName) -> Result<User> {
        self.users
            .remove(user_name)
            .ok_or(CommonError::UserExists(user_name.clone()))
    }

    pub fn get_user(&self, user_name: &UserName) -> Result<&User> {
        self.users
            .get(user_name)
            .ok_or(CommonError::UserNotExists(user_name.clone()))
    }

    pub fn list_users(&self) -> Vec<UserName> {
        self.users.keys().cloned().collect()
    }
}
