use crate::coder::Coder;
use crate::error::{Error::*, Result};
use arrayvec::ArrayVec;
use regr::Span;

/// Unicode code point inclusive range.
type UcpRange = std::ops::RangeInclusive<u32>;

const CODER_NAME: &str = "Utf8Coder";

pub struct Utf8Coder;

impl Coder for Utf8Coder {
    const MIN_CODEPOINT: u32 = char::MIN as u32;

    const MAX_CODEPOINT: u32 = char::MAX as u32;

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

    fn encode_range<F>(&self, start_ucp: u32, end_ucp: u32, handler: F) -> Result<()>
    where
        F: FnMut(&[Span]),
    {
        assert!(start_ucp <= end_ucp);
        _ = char_try_from(start_ucp)?;
        _ = char_try_from(end_ucp)?;
        let mut handler = handler;
        encode_range(start_ucp..=end_ucp, &mut handler)
    }

    fn encode_entire_range<F>(&self, handler: F) -> Result<()>
    where
        F: FnMut(&[Span]),
    {
        self.encode_range(Self::MIN_CODEPOINT, Self::MAX_CODEPOINT, handler)
    }
}

fn encode_range<F>(range: UcpRange, handler: &mut F) -> Result<()>
where
    F: FnMut(&[Span]),
{
    let mut range = range;
    while let Some((range, bytes_len)) = take_n_bytes_range(&mut range) {
        match bytes_len {
            1 => handle_range::<1>(*range.start(), *range.end(), handler),
            2 => handle_range::<2>(*range.start(), *range.end(), handler),
            3 => handle_range::<3>(*range.start(), *range.end(), handler),
            4 => handle_range::<4>(*range.start(), *range.end(), handler),
            _ => unreachable!(),
        }
    }
    Ok(())
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
            let start = *range.start().max(&0xE000); // skip surrogates
            let end = *range.end().min(&0xFFFF);
            *range = end + 1..=*range.end();
            Some((start..=end, 3))
        }
        0x10000..=0x10FFFF => {
            let start = *range.start();
            let end = *range.end().min(&0x10FFFF);
            *range = end + 1..=*range.end();
            Some((start..=end, 4))
        }
        _ => None,
    }
}

fn handle_range<const N: usize>(start: u32, end: u32, handler: &mut impl FnMut(&[Span])) {
    const MASK: u32 = 0x3F;
    const MASK_LEN: usize = 6;

    let mut start = start;
    let mut mask = 0;
    for _ in 0..N - 1 {
        mask <<= MASK_LEN;
        mask |= MASK;
        let part = start & mask;
        if part == 0 {
            continue;
        }
        let tmp_end = start | mask;
        if tmp_end > end {
            break;
        }
        run_handler::<N>(start, tmp_end, handler);
        start = tmp_end + 1;
    }

    let mut reversed_codepoints = ArrayVec::<(u32, u32), N>::new();
    let mut end = end;
    let mut mask = 0;
    for _ in 0..N - 1 {
        mask <<= MASK_LEN;
        mask |= MASK;
        let part = end & mask;
        if part == mask {
            continue;
        }
        let tmp_start = end & !mask;
        if tmp_start < start {
            break;
        }
        reversed_codepoints.push((tmp_start, end));
        end = tmp_start - 1;
    }

    if start <= end {
        run_handler::<N>(start, end, handler);
    }

    // to save ascending order of the range sequences
    for (start, end) in reversed_codepoints.iter().rev() {
        run_handler::<N>(*start, *end, handler);
    }
}

fn run_handler<const N: usize>(start: u32, end: u32, handler: &mut impl FnMut(&[Span])) {
    let mut start = start;
    let mut end = end;
    let mut buffer = [Span::new_unchecked(0, 0); N];
    for i in (1..N).rev() {
        let start_byte = (start as u8 & 0x3F) | 0x80;
        start >>= 6;
        let end_byte = (end as u8 & 0x3F) | 0x80;
        end >>= 6;
        buffer[i] = Span::new(start_byte, end_byte);
    }
    let prefix: u8 = match N {
        1 => 0x00,
        2 => 0xC0,
        3 => 0xE0,
        4 => 0xF0,
        _ => unreachable!("Invalid UTF-8 sequence length"),
    };
    buffer[0] = Span::new_unchecked(start as u8 | prefix, end as u8 | prefix);
    handler(&buffer);
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
