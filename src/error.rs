pub type Result<T> = std::result::Result<T, Error>;
//pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, derive_more::From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
    #[from]
    Connection(crate::connection::ConnectionError),
    #[from]
    Common(crate::common::CommonError),
    #[from]
    Server(crate::server::ServerError),
    #[from]
    Client(crate::client::ClientError),
}

//Error boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
