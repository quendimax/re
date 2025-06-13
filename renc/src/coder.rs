use crate::error::Result;
use regr::Span;

/// This trait helps convert unicode code points into byte sequeces
/// corresponding encoding way chosen by user.
pub trait Coder {
    /// Encode unicode code point into a byte sequence
    fn encode_ucp(&self, codepoint: u32, buffer: &mut [u8]) -> Result<usize>;

    /// Encode char into a byte sequence.
    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize> {
        self.encode_ucp(c as u32, buffer)
    }

    /// Encode string into a byte sequence.
    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize>;

    /// Encode range of unicode code points into array of byte sequences.
    fn encode_range(&self, start_ucp: u32, end_ucp: u32, handler: fn(&[Span])) -> Result<()>;
}
