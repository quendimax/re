use crate::encoding::Encoding;
use crate::error::Result;
use redt::Range;

/// This trait helps convert unicode code points into byte sequences
/// corresponding to the encoding way chosen by the user.
pub trait Encoder {
    fn encoding(&self) -> Encoding;

    /// Encode unicode code point into a byte sequence
    fn encode_ucp(&self, codepoint: u32, buffer: &mut [u8]) -> Result<usize>;

    /// Encode char into a byte sequence.
    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize>;

    /// Encode string into a byte sequence.
    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize>;

    /// Encode range of unicode code points into array of byte sequences.
    ///
    /// If input range contains invalid code points, the method should ignore
    /// them.
    fn encode_range<F>(&self, start_ucp: u32, end_ucp: u32, handler: F)
    where
        F: FnMut(&[Range<u8>]);

    /// Encode the entire range of code points allowed by this coder into array
    /// of byte sequences.
    fn encode_entire_range<F>(&self, handler: F)
    where
        F: FnMut(&[Range<u8>]);
}
