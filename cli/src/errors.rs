use failchain::{BoxedError, ChainErrorKind};
use failure::Fail;

pub type Error = BoxedError<ErrorKind>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "{}", 0)]
    Client(String),

    #[fail(display = "{}", 0)]
    Config(String),

    #[fail(display = "Input error: {}", 0)]
    Input(String),

    #[fail(display = "Unknown shell `{}`", 0)]
    UnknownShell(String),

    #[fail(display = "{}", 0)]
    UnknownOutputFormat(String),

    #[fail(display = "An unknown error has occurred: {}", 0)]
    Unknown(String),
}

impl ChainErrorKind for ErrorKind {
    type Error = Error;
}
