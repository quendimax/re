use crate::coder::Coder;
use crate::error::{Error::*, Result};

pub struct Utf8Coder;

impl Coder for Utf8Coder {
    const CODER_NAME: &'static str = "Utf8Coder";

    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize> {
        let expected_len = c.len_utf8();
        if buffer.len() < expected_len {
            Err(SmallBuffer)
        } else {
            c.encode_utf8(buffer);
            Ok(expected_len)
        }
    }

    fn encode_ucp(&self, code_point: u32, buffer: &mut [u8]) -> Result<usize> {
        if let Ok(c) = char::try_from(code_point) {
            self.encode_char(c, buffer)
        } else if code_point <= 0x10FFFF {
            Err(SurrogateUnsupported {
                coder_name: Self::CODER_NAME,
            })
        } else {
            Err(InvalidCodePoint(code_point))
        }
    }

    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize> {
        let expected_len = s.len();
        if buffer.len() < expected_len {
            Err(SmallBuffer)
        } else {
            buffer[..expected_len].copy_from_slice(s.as_bytes());
            Ok(expected_len)
        }
    }
}
