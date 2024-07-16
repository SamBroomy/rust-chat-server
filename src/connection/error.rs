pub type Result<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug, derive_more::From)]
pub enum ConnectionError {
    #[from]
    BincodeDecode(bincode::error::DecodeError),
    #[from]
    BincodeEncode(bincode::error::EncodeError),
    #[from]
    Io(std::io::Error),

    ConnectionDropped,
    InvalidFrameSize,
}

//Error boilerplate
impl core::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConnectionError {}
