mod error;
pub mod messages;
mod room;
mod user;

pub use error::CommonError;
use error::Result;

pub use room::{RoomManager, RoomName};
pub use user::UserName;
