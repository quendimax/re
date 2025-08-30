use std::ops::Range;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<ErrorKind>;

#[derive(Error, Debug, PartialEq)]
pub enum ErrorKind {
    #[error("encoder error: {cause}")]
    EncoderError {
        #[source]
        cause: renc::Error,
        span: Range<usize>,
    },

    #[error("expected {expected}, but found `{found_spell}`")]
    Unexpected {
        found_spell: Box<str>,
        found_span: Range<usize>,
        expected: Box<str>,
    },

    #[error("value `{value}` is out of {range}")]
    OutOfRange {
        value: Box<str>,
        span: Range<usize>,
        range: Box<str>,
    },

    #[error("empty escape expression is not allowed")]
    EmptyEscape { span: Range<usize> },

    #[error("unsupported escape sequence `{sequence}`")]
    UnsupportedEscape {
        sequence: Box<str>,
        span: Range<usize>,
    },

    #[error("zero repetition `{{0,0}}` is not allowed")]
    ZeroRepetition { span: Range<usize> },

    #[error("repetition expression `{{n,m}}` expects that `n <= m`")]
    InvalidRepetition { span: Range<usize> },
}

impl ErrorKind {
    pub fn error_span(&self) -> Range<usize> {
        use ErrorKind::*;
        match self {
            EncoderError { span, .. } => span.clone(),
            Unexpected { found_span, .. } => found_span.clone(),
            OutOfRange { span, .. } => span.clone(),
            EmptyEscape { span } => span.clone(),
            UnsupportedEscape { span, .. } => span.clone(),
            ZeroRepetition { span } => span.clone(),
            InvalidRepetition { span } => span.clone(),
        }
    }
}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::*;

    pub(crate) fn encoder_error<T>(cause: renc::Error, span: Range<usize>) -> Result<T> {
        Err(Box::new(ErrorKind::EncoderError { cause, span }))
    }

    pub(crate) fn unexpected<T>(
        found_spell: impl Into<Box<str>>,
        found_span: Range<usize>,
        expected: impl Into<Box<str>>,
    ) -> Result<T> {
        Err(Box::new(ErrorKind::Unexpected {
            found_spell: found_spell.into(),
            found_span,
            expected: expected.into(),
        }))
    }

    pub(crate) fn out_of_range<T>(
        value: impl Into<Box<str>>,
        span: Range<usize>,
        range: impl Into<Box<str>>,
    ) -> Result<T> {
        Err(Box::new(ErrorKind::OutOfRange {
            value: value.into(),
            span,
            range: range.into(),
        }))
    }

    pub(crate) fn empty_escape<T>(span: Range<usize>) -> Result<T> {
        Err(Box::new(ErrorKind::EmptyEscape { span }))
    }

    pub(crate) fn unsupported_escape<T, S>(sequence: S, span: Range<usize>) -> Result<T>
    where
        S: Into<Box<str>>,
    {
        Err(Box::new(ErrorKind::UnsupportedEscape {
            sequence: sequence.into(),
            span,
        }))
    }

    pub(crate) fn zero_repetition<T>(span: Range<usize>) -> Result<T> {
        Err(Box::new(ErrorKind::ZeroRepetition { span }))
    }

    pub(crate) fn invalid_repetition<T>(span: Range<usize>) -> Result<T> {
        Err(Box::new(ErrorKind::InvalidRepetition { span }))
    }
}
