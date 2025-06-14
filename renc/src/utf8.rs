use crate::coder::Coder;
use crate::error::{Error::*, Result};
use regr::{Span, span};
use std::ops::RangeInclusive;

/// Unicode code point inclusive range.
type UcpRange = std::ops::RangeInclusive<u32>;

const CODER_NAME: &str = "Utf8Coder";

pub struct Utf8Coder;

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

    fn encode_range<F>(&self, ucp_range: RangeInclusive<u32>, handler: F) -> Result<()>
    where
        F: FnMut(&[Span]),
    {
        let start_ucp = *ucp_range.start();
        let end_ucp = *ucp_range.end();
        assert!(start_ucp <= end_ucp);
        _ = char_try_from(start_ucp)?;
        _ = char_try_from(end_ucp)?;
        let mut handler = handler;
        encode_range(start_ucp..=end_ucp, &mut handler)
    }
}

fn encode_range<F>(range: UcpRange, handler: &mut F) -> Result<()>
where
    F: FnMut(&[Span]),
{
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

fn encode_known_range<const N: usize>(range: UcpRange, handler: &mut impl FnMut(&[Span])) {
    dbg!("encode_kown_range", &range);
    let mut start_buffer = [0u8; N];
    let mut end_buffer = [0u8; N];
    let start_str = char::from_u32(*range.start())
        .unwrap()
        .encode_utf8(&mut start_buffer);
    let end_str = char::from_u32(*range.end())
        .unwrap()
        .encode_utf8(&mut end_buffer);
    assert_eq!(start_str.len(), N);
    assert_eq!(end_str.len(), N);

    handle_sequence(&start_buffer, &end_buffer, true, 0, handler);
}

fn handle_sequence<const N: usize>(
    start_bytes: &[u8; N],
    end_bytes: &[u8; N],
    all_previous_are_equal: bool,
    index: usize,
    handler: &mut impl FnMut(&[Span]),
) {
    let start = format!("{:X?}", start_bytes);
    let end = format!("{:X?}", end_bytes);
    dbg!("--------enter-handle_sequence", index, start, end);
    if index >= N {
        dbg!("---print");
        run_handler(start_bytes, end_bytes, handler);
        dbg!("--------exit-handle_sequence");
        return;
    }
    if all_previous_are_equal {
        if start_bytes[index] == end_bytes[index] {
            dbg!("-1");
            handle_sequence(start_bytes, end_bytes, true, index + 1, handler);
        } else {
            dbg!("0");
            handle_sequence(start_bytes, end_bytes, false, index + 1, handler);
        }
    } else {
        debug_assert!(index > 0);

        if start_bytes[index] == 0x80 && end_bytes[index] == 0xBF {
            dbg!("1");
            handle_sequence(start_bytes, end_bytes, false, index + 1, handler);
        } else if start_bytes[index - 1] + 1 == end_bytes[index - 1] {
            dbg!("2");
            let mut start_mid_bytes = *start_bytes;
            set_tail(&mut start_mid_bytes, 0xBF, index);
            let equal = start_bytes[index] == start_mid_bytes[index];
            handle_sequence(start_bytes, &start_mid_bytes, equal, index + 1, handler);

            let mut end_mid_bytes = *end_bytes;
            set_tail(&mut end_mid_bytes, 0x80, index);
            let equal = end_mid_bytes[index] == end_bytes[index];
            handle_sequence(&end_mid_bytes, end_bytes, equal, index + 1, handler);
        } else if start_bytes[index] == 0x80 {
            dbg!("3");
            let mut start_mid_bytes = *end_bytes;
            decrement_from_index(&mut start_mid_bytes, index - 1);
            set_tail(&mut start_mid_bytes, 0xBF, index);
            handle_sequence(start_bytes, &start_mid_bytes, false, index + 1, handler);

            let mut end_mid_bytes = *end_bytes;
            set_tail(&mut end_mid_bytes, 0x80, index);
            let equal = end_mid_bytes[index] == end_bytes[index];
            handle_sequence(&end_mid_bytes, end_bytes, equal, index + 1, handler);
        } else if end_bytes[index] == 0xBF {
            dbg!("4");
            let mut start_mid_bytes = *start_bytes;
            set_tail(&mut start_mid_bytes, 0xBF, index);
            let equal = start_bytes[index] == start_mid_bytes[index];
            handle_sequence(start_bytes, &start_mid_bytes, equal, index + 1, handler);

            let mut end_mid_bytes = *start_bytes;
            increment_from_index(&mut end_mid_bytes, index - 1);
            set_tail(&mut end_mid_bytes, 0x80, index);
            handle_sequence(&end_mid_bytes, end_bytes, false, index + 1, handler);
        } else {
            dbg!("5");
            let mut start_mid_bytes = *start_bytes;
            set_tail(&mut start_mid_bytes, 0xBF, index);
            let equal = start_bytes[index] == start_mid_bytes[index];
            handle_sequence(start_bytes, &start_mid_bytes, equal, index + 1, handler);

            let mut mid_start_bytes = *start_bytes;
            increment_from_index(&mut mid_start_bytes, index - 1);
            set_tail(&mut mid_start_bytes, 0x80, index);
            let mut mid_end_bytes = *end_bytes;
            decrement_from_index(&mut mid_end_bytes, index - 1);
            set_tail(&mut mid_end_bytes, 0xBF, index);
            handle_sequence(&mid_start_bytes, &mid_end_bytes, false, index + 1, handler);

            let mut end_mid_bytes = *end_bytes;
            set_tail(&mut end_mid_bytes, 0x80, index);
            let equal = end_mid_bytes[index] == end_bytes[index];
            handle_sequence(&end_mid_bytes, end_bytes, equal, index + 1, handler);
        }
    }
    dbg!("--------exit-handle_sequence");
}

fn set_tail<const N: usize>(bytes: &mut [u8; N], value: u8, from_index: usize) {
    for byte in &mut bytes[from_index..] {
        *byte = value;
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

fn take_n_bytes_range(range: &mut UcpRange) -> Option<(UcpRange, usize)> {
    if range.start() > range.end() {
        return None;
    }
    match *range.start() {
        0..=0x7F => {
            let start = *range.start();
            let end = *range.end().min(&0x7F);
            *range = end + 1..=*range.end();
            Some((start..=end, 1))
        }
        0x80..=0x7FF => {
            let start = *range.start();
            let end = *range.end().min(&0x7FF);
            *range = end + 1..=*range.end();
            Some((start..=end, 2))
        }
        0x800..=0xD7FF => {
            let start = *range.start();
            let end = *range.end().min(&0xD7FF);
            *range = end + 1..=*range.end();
            Some((start..=end, 3))
        }
        0xD800..=0xFFFF => {
            let start = *range.start().max(&0xE000);
            let end = *range.end().min(&0xFFFF);
            *range = end + 1..=*range.end();
            Some((start..=end, 3))
        }
        0x10000..=0x10FFFF => {
            let start = *range.start();
            let end = *range.end().min(&0x7F);
            *range = end + 1..=*range.end();
            Some((start..=end, 4))
        }
        _ => None,
    }
}

fn run_handler(start_bytes: &[u8], end_bytes: &[u8], handler: &mut impl FnMut(&[Span])) {
    match start_bytes.len() {
        1 => {
            handler(&[span(start_bytes[0]..=end_bytes[0])]);
        }
        2 => {
            handler(&[
                span(start_bytes[0]..=end_bytes[0]),
                span(start_bytes[1]..=end_bytes[1]),
            ]);
        }
        3 => {
            handler(&[
                span(start_bytes[0]..=end_bytes[0]),
                span(start_bytes[1]..=end_bytes[1]),
                span(start_bytes[2]..=end_bytes[2]),
            ]);
        }
        4 => {
            handler(&[
                span(start_bytes[0]..=end_bytes[0]),
                span(start_bytes[1]..=end_bytes[1]),
                span(start_bytes[2]..=end_bytes[2]),
                span(start_bytes[3]..=end_bytes[3]),
            ]);
        }
        _ => unreachable!(),
    }
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
