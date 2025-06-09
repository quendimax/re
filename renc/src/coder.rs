use crate::error::CoderError;

/// This trait helps convert unicode code points into byte sequeces
/// corresponding encoding way chosen by user.
pub trait Coder {
    const CODER_NAME: &'static str;

    /// Encode unicode code point into a byte sequence
    fn encode_ucp(&self, codepoint: u32, buffer: &mut [u8]) -> Result<usize, CoderError>;

    /// Encode char into a byte sequence.
    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize, CoderError> {
        self.encode_ucp(c as u32, buffer)
    }

    /// Encode string into a byte sequence.
    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize, CoderError>;
}
