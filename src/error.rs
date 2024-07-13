pub type Result<T> = std::result::Result<T, Error>;
//pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, derive_more::From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
}

//Error boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self)
    }
}

impl std::error::Error for Error {}
