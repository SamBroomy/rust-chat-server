use bincode::{Decode, Encode};
use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
};

#[derive(Debug, Clone, Encode, Decode, PartialEq, Hash, Eq)]
pub struct UserName {
    username: String,
}

impl UserName {
    pub fn new(username: String) -> Self {
        Self { username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

impl PartialEq<&str> for UserName {
    fn eq(&self, other: &&str) -> bool {
        self.username == *other
    }
}

impl PartialEq<String> for UserName {
    fn eq(&self, other: &String) -> bool {
        self.username == *other
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
