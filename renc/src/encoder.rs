use crate::error::Result;
use redt::Range;

/// This trait helps convert unicode code points into byte sequeces
/// corresponding encoding way chosen by user.
pub trait Encoder {
    /// Minimum code point that can be encoded by this coder.
    const MIN_CODE_POINT: u32;

    /// Maximum code point that can be encoded by this coder.
    const MAX_CODE_POINT: u32;

    /// Encode unicode code point into a byte sequence
    fn encode_ucp(&self, codepoint: u32, buffer: &mut [u8]) -> Result<usize>;

    /// Encode char into a byte sequence.
    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize>;

    /// Encode string into a byte sequence.
    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize>;

    /// Encode range of unicode code points into array of byte sequences.
    fn encode_range<F>(&self, start_ucp: u32, end_ucp: u32, handler: F) -> Result<()>
    where
        F: FnMut(&[Range<u8>]);

    /// Encode the entire range of code points allowed by this coder into array
    /// of byte sequences.
    fn encode_entire_range<F>(&self, handler: F) -> Result<()>
    where
        F: FnMut(&[Range<u8>]);
}
