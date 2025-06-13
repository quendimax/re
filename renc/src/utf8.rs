use crate::coder::Coder;
use crate::error::{Error::*, Result};
use crate::range::Range;

pub struct Utf8Coder;

const CODER_NAME: &str = "Utf8Coder";

impl Coder for Utf8Coder {
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
        let c = char_try_from(code_point)?;
        self.encode_char(c, buffer)
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

    fn encode_range(&self, range: Range<u32>, handler: fn(byte_seq: &[Range<u8>])) -> Result<()> {
        assert!(range.start <= range.end);
        _ = char_try_from(range.start)?;
        _ = char_try_from(range.end)?;
        encode_range(range, handler)
    }
}

fn encode_range(range: Range<u32>, handler: fn(byte_seq: &[Range<u8>])) -> Result<()> {
    let mut range = range;
    while let Some((range, bytes_len)) = take_n_bytes_range(&mut range) {
        match bytes_len {
            1 => encode_known_range::<1>(range, handler),
            2 => encode_known_range::<2>(range, handler),
            3 => encode_known_range::<3>(range, handler),
            4 => encode_known_range::<4>(range, handler),
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn encode_known_range<const N: usize>(range: Range<u32>, handler: fn(byte_seq: &[Range<u8>])) {
    let mut start_buffer = [0u8; N];
    let mut end_buffer = [0u8; N];
    let start_str = char::from_u32(range.start)
        .unwrap()
        .encode_utf8(&mut start_buffer);
    let end_str = char::from_u32(range.end)
        .unwrap()
        .encode_utf8(&mut end_buffer);
    assert_eq!(start_str.len(), N);
    assert_eq!(end_str.len(), N);

    handle_sequence(&mut start_buffer, &mut end_buffer, true, 0, handler);
}

fn handle_sequence<const N: usize>(
    start_bytes: &mut [u8; N],
    end_bytes: &mut [u8; N],
    all_previous_are_equal: bool,
    index: usize,
    handler: fn(byte_seq: &[Range<u8>]),
) {
    if index >= N {
        run_handler(start_bytes, end_bytes, handler);
        return;
    }
    if all_previous_are_equal {
        if start_bytes[index] == end_bytes[index] {
            handle_sequence(start_bytes, end_bytes, true, index + 1, handler);
        } else {
            handle_sequence(start_bytes, end_bytes, false, index + 1, handler);
        }
    } else {
        debug_assert!(index > 0);

        let mut old_start_bytes = *start_bytes;
        start_bytes[index] = 0xBF;
        let all_previous_are_equal =
            all_previous_are_equal && (start_bytes[index] == old_start_bytes[index]);
        handle_sequence(
            &mut old_start_bytes,
            start_bytes,
            all_previous_are_equal,
            index + 1,
            handler,
        );
        increment_from_index(start_bytes, index);

        let mut old_end_bytes = *end_bytes;
        end_bytes[index] = 0x80;
        let all_previous_are_equal =
            all_previous_are_equal && (end_bytes[index] == old_end_bytes[index]);
        handle_sequence(
            end_bytes,
            &mut old_end_bytes,
            all_previous_are_equal,
            index + 1,
            handler,
        );
        decrement_from_index(end_bytes, index);

        handle_sequence(
            start_bytes,
            end_bytes,
            all_previous_are_equal && (start_bytes[index] == end_bytes[index]),
            index + 1,
            handler,
        );
    }
}

fn increment_from_index<const N: usize>(utf8_seq: &mut [u8; N], index: usize) {
    if utf8_seq[index] == 0xBF {
        utf8_seq[index] = 0x80;
        increment_from_index(utf8_seq, index - 1);
    } else {
        utf8_seq[index] += 1;
    }
}

fn decrement_from_index<const N: usize>(utf8_seq: &mut [u8; N], index: usize) {
    if utf8_seq[index] == 0x80 {
        utf8_seq[index] = 0xBF;
        decrement_from_index(utf8_seq, index - 1);
    } else {
        utf8_seq[index] -= 1;
    }
}

fn take_n_bytes_range(range: &mut Range<u32>) -> Option<(Range<u32>, usize)> {
    if range.start > range.end {
        return None;
    }
    match range.start {
        0..=0x7F => {
            let start = range.start;
            let end = range.end.min(0x7F);
            range.start = end + 1;
            Some((rng(start, end), 1))
        }
        0x80..=0x7FF => {
            let start = range.start;
            let end = range.end.min(0x7FF);
            range.start = end + 1;
            Some((rng(start, end), 2))
        }
        0x800..=0xD7FF => {
            let start = range.start;
            let end = range.end.min(0xD7FF);
            range.start = end + 1;
            Some((rng(start, end), 3))
        }
        0xD800..=0xFFFF => {
            let start = range.start.max(0xE000);
            let end = range.end.min(0xFFFF);
            range.start = end + 1;
            Some((rng(start, end), 3))
        }
        0x10000..=0x10FFFF => {
            let start = range.start;
            let end = range.end.min(0x7F);
            range.start = end + 1;
            Some((rng(start, end), 4))
        }
        _ => None,
    }
}

fn run_handler(start_bytes: &[u8], end_bytes: &[u8], handler: fn(byte_seq: &[Range<u8>])) {
    match start_bytes.len() {
        1 => {
            handler(&[rng(start_bytes[0], end_bytes[0])]);
        }
        2 => {
            handler(&[
                rng(start_bytes[0], end_bytes[0]),
                rng(start_bytes[1], end_bytes[1]),
            ]);
        }
        3 => {
            handler(&[
                rng(start_bytes[0], end_bytes[0]),
                rng(start_bytes[1], end_bytes[1]),
                rng(start_bytes[2], end_bytes[2]),
            ]);
        }
        4 => {
            handler(&[
                rng(start_bytes[0], end_bytes[0]),
                rng(start_bytes[1], end_bytes[1]),
                rng(start_bytes[2], end_bytes[2]),
                rng(start_bytes[3], end_bytes[3]),
            ]);
        }
        _ => unreachable!(),
    }
}

fn rng<T>(start: T, end: T) -> Range<T> {
    Range { start, end }
}

fn char_try_from(codepoint: u32) -> Result<char> {
    if let Ok(c) = char::try_from(codepoint) {
        Ok(c)
    } else if codepoint <= 0x10FFFF {
        Err(SurrogateUnsupported {
            coder_name: CODER_NAME,
        })
    } else {
        Err(InvalidCodePoint(codepoint))
    }
}
