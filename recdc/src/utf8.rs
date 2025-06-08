use crate::codec::Codec;
use crate::error::CodecError::{self, *};

pub struct Utf8Codec;

impl Codec for Utf8Codec {
    const CODEC_NAME: &'static str = "Utf8Codec";

    fn encode_char(&self, c: char, buffer: &mut [u8]) -> Result<usize, CodecError> {
        let expected_len = c.len_utf8();
        if buffer.len() < expected_len {
            Err(SmallBuffer)
        } else {
            c.encode_utf8(buffer);
            Ok(expected_len)
        }
    }

    fn encode_ucp(&self, code_point: u32, buffer: &mut [u8]) -> Result<usize, CodecError> {
        if let Ok(c) = char::try_from(code_point) {
            self.encode_char(c, buffer)
        } else if code_point <= 0x10FFFF {
            Err(SurrogateUnsupported {
                codec_name: Self::CODEC_NAME,
            })
        } else {
            Err(InvalidCodePoint(code_point))
        }
    }

    fn encode_str(&self, s: &str, buffer: &mut [u8]) -> Result<usize, CodecError> {
        let expected_len = s.len();
        if buffer.len() < expected_len {
            Err(SmallBuffer)
        } else {
            buffer[..expected_len].copy_from_slice(s.as_bytes());
            Ok(expected_len)
        }
    }
}
