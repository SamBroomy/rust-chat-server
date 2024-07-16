use bincode::{Decode, Encode};
use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
};

#[derive(Debug, Clone, Encode, Decode, PartialEq, Hash, Eq)]
pub struct User {
    username: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self { username }
    }
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl PartialEq<&str> for User {
    fn eq(&self, other: &&str) -> bool {
        self.username == *other
    }
}

impl PartialEq<String> for User {
    fn eq(&self, other: &String) -> bool {
        self.username == *other
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
