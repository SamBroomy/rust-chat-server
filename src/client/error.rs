pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, derive_more::From)]
pub enum ClientError {
    #[from]
    Connection(crate::connection::ConnectionError),
    #[from]
    Io(std::io::Error),

    InvalidCommand,
}

//Error boilerplate
impl core::fmt::Display for ClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ClientError {}
