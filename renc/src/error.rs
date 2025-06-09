use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CoderError {
    /// Error returned when a surrogate code point is encountered in an encoding
    /// operation.
    ///
    /// Surrogate code points are code points in the range U+D800 to U+DFFF,
    /// which are reserved for use in UTF-16 and cannot be represented in some
    /// encodings.
    ///
    /// # Parameters
    ///
    /// - `codec_name`: The encoding that encountered the surrogate code point.
    #[error("surrogate code points are not supported by {coder_name}")]
    SurrogateUnsupported { coder_name: &'static str },

    /// Error returned when the provided output buffer is too small to hold the
    /// encoded byte sequence.
    #[error("output buffer for the encoded byte sequence is too small")]
    SmallBuffer,

    /// Error returned when an invalid Unicode code point is encountered.
    ///
    /// This error occurs when a code point is outside the valid Unicode range
    /// (U+0000 to U+10FFFF).
    ///
    /// This error doesn't evolved for surrogate code points. Look at
    /// [`CodecError::SurrogateUnsupported`] for that.
    ///
    /// # Parameters
    ///
    /// - `0`: The invalid code point value.
    #[error("invalid unicode code point '\\x{0:X}'")]
    InvalidCodePoint(u32),
}
