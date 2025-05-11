mod error;
mod utf8;

pub use error::CodecError;

/// This trait helps convert unicode code points into byte sequeces
/// corresponding encoding way chosen by user.
pub trait Codec {
    const CODEC_NAME: &'static str;

    /// Encode unicode code point int oa byte sequence
    fn encode_ucp(code_point: u32, buffer: &mut [u8]) -> Result<usize, CodecError>;

    /// Encode char into a byte sequence.
    fn encode_char(c: char, buffer: &mut [u8]) -> Result<usize, CodecError> {
        Self::encode_ucp(c as u32, buffer)
    }

    /// Encode string into a byte sequence.
    fn encode_str(s: &str, buffer: &mut [u8]) -> Result<usize, CodecError>;
}
