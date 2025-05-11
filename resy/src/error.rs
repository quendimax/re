use crate::codec::CodecError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("escape expression '\\{0}' is not supported")]
    UnsupportedEscape(char),

    #[error("unexpected end of file: {0}")]
    UnexpectedEof(&'static str),

    #[error("codec error: {0}")]
    CodecError(#[from] CodecError),
}
