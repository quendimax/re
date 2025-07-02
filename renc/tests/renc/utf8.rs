use arrayvec::ArrayVec;
use assert_matches::assert_matches;
use pretty_assertions::assert_eq;
use regex_syntax::utf8::Utf8Sequences;
use regr::{Span, span};
use renc::{Coder, Error, Result, Utf8Coder};
use std::ops::RangeInclusive;

static CODER: Utf8Coder = Utf8Coder;

type Sequence = ArrayVec<Span, 4>;

// CAP == 11 found with bruteforcing all utf8 range sequences
type Sequences = ArrayVec<Sequence, 11>;

fn arr<T: Clone, const CAP: usize>(spans: &[T]) -> ArrayVec<T, CAP> {
    ArrayVec::<T, CAP>::try_from(spans).unwrap()
}

fn encode_range(range: RangeInclusive<u32>) -> Result<Sequences> {
    let mut seq = Sequences::new();
    let start = *range.start();
    let end = *range.end();
    Utf8Coder.encode_range(start, end, |spans| {
        seq.push(Sequence::try_from(spans).unwrap())
    })?;
    Ok(seq)
}

fn _encode<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<Sequences> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    encode_range(start..=end)
}

fn expect_range(range: RangeInclusive<u32>) -> Result<Sequences> {
    let start = char::from_u32(*range.start()).unwrap();
    let end = char::from_u32(*range.end()).unwrap();
    let seq = Utf8Sequences::new(start, end)
        .map(|s| {
            s.into_iter()
                .map(|r| Span::new(r.start, r.end))
                .collect::<Sequence>()
        })
        .collect::<Sequences>();
    Ok(seq)
}

fn _expect<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<Sequences> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    expect_range(start..=end)
}

#[test]
fn encode_char() {
    let mut buffer = [0u8; 4];
    assert_eq!(CODER.encode_char('a', &mut buffer), Ok(1));
    assert_eq!(buffer, [b'a', 0, 0, 0]);

    assert_eq!(CODER.encode_char('ў', &mut buffer), Ok(2));
    assert_eq!(buffer, [0xD1, 0x9E, 0, 0]);

    assert_eq!(CODER.encode_char('Ⲁ', &mut buffer), Ok(3));
    assert_eq!(buffer, [0xE2, 0xB2, 0x80, 0]);

    assert_eq!(CODER.encode_char('𐌰', &mut buffer), Ok(4));
    assert_eq!(buffer, [0xF0, 0x90, 0x8C, 0xB0]);
}

#[test]
fn encode_char_fails() {
    let mut buffer = [0u8; 2];
    assert_eq!(CODER.encode_char('𐌰', &mut buffer), Err(Error::SmallBuffer));
}

#[test]
fn encode_ucp() {
    let mut buffer = [0u8; 4];
    assert_eq!(CODER.encode_ucp('a' as u32, &mut buffer), Ok(1));
    assert_eq!(buffer, [b'a', 0, 0, 0]);

    assert_eq!(CODER.encode_ucp('ў' as u32, &mut buffer), Ok(2));
    assert_eq!(buffer, [0xD1, 0x9E, 0, 0]);

    assert_eq!(CODER.encode_ucp('Ⲁ' as u32, &mut buffer), Ok(3));
    assert_eq!(buffer, [0xE2, 0xB2, 0x80, 0]);

    assert_eq!(CODER.encode_ucp('𐌰' as u32, &mut buffer), Ok(4));
    assert_eq!(buffer, [0xF0, 0x90, 0x8C, 0xB0]);
}

#[test]
fn encode_ucp_fails() {
    let mut buffer = [0u8; 3];
    assert_eq!(
        CODER.encode_ucp('𐌰' as u32, &mut buffer),
        Err(Error::SmallBuffer)
    );
    assert_eq!(
        CODER.encode_ucp(0x110000, &mut buffer),
        Err(Error::InvalidCodePoint(0x110000))
    );
    assert_matches!(
        CODER.encode_ucp(0xD811, &mut buffer),
        Err(Error::SurrogateUnsupported { .. })
    );
}

#[test]
fn encode_str() {
    let mut buffer = [0u8; 9];
    assert_eq!(CODER.encode_str("abc", &mut buffer), Ok(3));
    assert_eq!(&buffer[..3], [b'a', b'b', b'c']);

    assert_eq!(CODER.encode_str("ўⲀ𐌰", &mut buffer), Ok(9));
    assert_eq!(
        buffer,
        [0xD1, 0x9E, 0xE2, 0xB2, 0x80, 0xF0, 0x90, 0x8C, 0xB0]
    );
}

#[test]
fn encode_str_fails() {
    let mut buffer = [0u8; 8];
    assert_eq!(
        CODER.encode_str("ўⲀ𐌰", &mut buffer),
        Err(Error::SmallBuffer)
    );
}

#[test]
fn encode_one_byte_ranges() {
    assert_eq!(encode_range(0..=0), Ok(arr(&[arr(&[span(0..=0)])])));
    assert_eq!(encode_range(0..=23), Ok(arr(&[arr(&[span(0..=23)])])));
    assert_eq!(encode_range(0..=0x7F), Ok(arr(&[arr(&[span(0..=0x7F)])])));
}

#[test]
fn encode_two_byte_ranges() {
    assert_eq!(
        encode_range(0x80..=0x81),
        Ok(arr(&[arr(&[
            span(0b110_00010..=0b110_00010),
            span(0b10_000000..=0b10_000001)
        ])]))
    );
    assert_eq!(
        encode_range(0x83..=0x734),
        Ok(arr(&[
            arr(&[
                span(0b110_00010..=0b110_00010),
                span(0b10_000011..=0b10_111111)
            ]),
            arr(&[
                span(0b110_00011..=0b110_11011),
                span(0b10_000000..=0b10_111111)
            ]),
            arr(&[
                span(0b110_11100..=0b110_11100),
                span(0b10_000000..=0b10_110100)
            ]),
        ]))
    );
    assert_eq!(
        encode_range(0x83..=0xD6),
        Ok(arr(&[
            arr(&[
                span(0b110_00010..=0b110_00010),
                span(0b10_000011..=0b10_111111)
            ]),
            arr(&[
                span(0b110_00011..=0b110_00011),
                span(0b10_000000..=0b10_010110)
            ]),
        ]))
    );
    assert_eq!(
        encode_range(0x80..=0x7FF),
        Ok(arr(&[arr(&[
            span(0b110_00010..=0b110_11111),
            span(0b10_000000..=0b10_111111)
        ])]))
    );
}

#[test]
fn encode_three_byte_ranges() {
    assert_eq!(encode_range(0x800..=0xFFFF), expect_range(0x800..=0xFFFF));
    assert_eq!(encode_range(0x800..=0x800), expect_range(0x800..=0x800));
}

#[test]
fn encode_four_byte_ranges() {
    assert_eq!(
        encode_range(0x10000..=0x10000),
        expect_range(0x10000..=0x10000)
    );
    assert_eq!(
        encode_range(0x10000..=0x10FFFF),
        expect_range(0x10000..=0x10FFFF)
    );
    assert_eq!(
        encode_range(0x10FFFF..=0x10FFFF),
        expect_range(0x10FFFF..=0x10FFFF)
    );
}

#[test]
fn encode_out_ranges() {
    assert_eq!(
        CODER.encode_range(0x800, 0x11FFFF, |_| {}),
        Err(Error::InvalidCodePoint(0x11FFFF))
    );
    assert_eq!(
        CODER.encode_range(0x118000, 0x11FFFF, |_| {}),
        Err(Error::InvalidCodePoint(0x118000))
    );
}

#[test]
fn encode_entire_range() {
    let mut seq = Sequences::new();
    assert!(
        Utf8Coder
            .encode_entire_range(|spans| seq.push(arr(spans)))
            .is_ok()
    );
    seq.sort();
    assert_eq!(Ok(seq.clone()), expect_range(0x0..=0x10FFFF));
    assert_eq!(Ok(seq), encode_range(0x0..=0x10FFFF));
}

mod prop {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;

    prop_compose! {
        fn gen_range()(mut start in 0..=0x10FFFFu32, mut end in 0..=0x10FFFFu32) -> (u32, u32) {
            if char::try_from(start).is_err() {
                start = 0xD7FF
            }
            if char::try_from(end).is_err() {
                end = 0xE000;
            }
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            (start, end)
        }
    }

    proptest! {
        #[test]
        fn encode_ranges((start, end) in gen_range()) {
            assert_eq!(encode_range(start..=end), expect_range(start..=end));
        }
    }
}

mod bruteforce {
    use pretty_assertions::assert_eq;
    use test_case::test_matrix;

    #[test_matrix(0x0..0x40)]
    #[cfg_attr(not(feature = "test-bruteforce-utf8"), ignore)]
    fn encode_range(iteration: u32) {
        const ITERATION_LEN: u32 = 0x40; // must be the same as in #[test_matrix]
        const CODEPOINT_NUM: u32 = 0x110000;
        const ITER_CHUNK_LEN: u32 = CODEPOINT_NUM / ITERATION_LEN;

        let first_cp = iteration * (ITER_CHUNK_LEN / 2);
        let first_char: char = char::try_from(first_cp).unwrap_or('\u{E000}');

        let last_cp = (iteration + 1) * (ITER_CHUNK_LEN / 2) - 1;
        let last_char: char = char::try_from(last_cp).unwrap_or('\u{D7FF}');

        if first_char <= last_char {
            for start in first_char..=last_char {
                for end in start..='\u{10FFFF}' {
                    assert_eq!(
                        super::encode_range(start as u32..=end as u32),
                        super::expect_range(start as u32..=end as u32),
                        "range: [{start:X?}-{end:X?}]",
                    );
                }
            }
        }

        let first_cp = CODEPOINT_NUM - (iteration + 1) * (ITER_CHUNK_LEN / 2);
        let first_char: char = char::try_from(first_cp).unwrap_or('\u{E000}');

        let last_cp = CODEPOINT_NUM - iteration * (ITER_CHUNK_LEN / 2) - 1;
        let last_char: char = char::try_from(last_cp).unwrap_or('\u{D7FF}');

        if first_char <= last_char {
            for start in first_char..=last_char {
                for end in start..='\u{10FFFF}' {
                    assert_eq!(
                        super::encode_range(start as u32..=end as u32),
                        super::expect_range(start as u32..=end as u32),
                        "range: [{start:X?}-{end:X?}]",
                    );
                }
            }
        }
    }
}
