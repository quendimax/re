use crate::codec::CodecError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("escape expression '\\{0}' is not supported")]
    UnsupportedEscape(char),

    #[error("empty escape expression is not supported")]
    EmptyEscape,

    #[error("unexpected end of file within {aborted_expr} expression")]
    UnexpectedEof { aborted_expr: String },

    #[error("expected token `{expected}`, but got `{unexpected}`")]
    UnexpectedToken {
        unexpected: String,
        expected: String,
    },

    #[error("unexpected close paren `)` encountered without open one")]
    UnexpectedCloseParen,

    #[error("codec error: {0}")]
    CodecError(#[from] CodecError),

    #[error("invalid hexadecimal {0}")]
    InvalidHex(String),

    #[error("value `{value}` is out of allowable range {range}")]
    OutOfRange { value: String, range: String },
}
