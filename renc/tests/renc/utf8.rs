use pretty_assertions::assert_eq;
use regex_syntax::utf8::Utf8Sequences;
use regr::{Span, span};
use renc::{Coder, Result, Utf8Coder};
use std::ops::RangeInclusive;

type Sequence = arrayvec::ArrayVec<Span, 4>;

fn arr(spans: &[Span]) -> Sequence {
    Sequence::try_from(spans).unwrap()
}

fn encode_range(range: RangeInclusive<u32>) -> Result<Vec<Sequence>> {
    let mut seq = Vec::<Sequence>::new();
    let start = *range.start();
    let end = *range.end();
    Utf8Coder.encode_range(start, end, |spans| {
        seq.push(Sequence::try_from(spans).unwrap())
    })?;
    seq.sort();
    Ok(seq)
}

fn _encode<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<Vec<Sequence>> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    encode_range(start..=end)
}

fn expect_range(range: RangeInclusive<u32>) -> Result<Vec<Sequence>> {
    let start = char::from_u32(*range.start()).unwrap();
    let end = char::from_u32(*range.end()).unwrap();
    let seq = Utf8Sequences::new(start, end)
        .map(|s| {
            s.into_iter()
                .map(|r| Span::new(r.start, r.end))
                .collect::<Sequence>()
        })
        .collect::<Vec<_>>();
    Ok(seq)
}

fn _expect<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<Vec<Sequence>> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    expect_range(start..=end)
}

#[test]
fn encode_one_byte_ranges() {
    assert_eq!(encode_range(0..=0), Ok(vec![arr(&[span(0..=0)])]));
    assert_eq!(encode_range(0..=23), Ok(vec![arr(&[span(0..=23)])]));
    assert_eq!(encode_range(0..=0x7F), Ok(vec![arr(&[span(0..=0x7F)])]));
}

#[test]
fn encode_two_byte_ranges() {
    assert_eq!(
        encode_range(0x80..=0x81),
        Ok(vec![arr(&[
            span(0b110_00010..=0b110_00010),
            span(0b10_000000..=0b10_000001)
        ])])
    );
    assert_eq!(
        encode_range(0x83..=0x734),
        Ok(vec![
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
        ])
    );
    assert_eq!(
        encode_range(0x83..=0xD6),
        Ok(vec![
            arr(&[
                span(0b110_00010..=0b110_00010),
                span(0b10_000011..=0b10_111111)
            ]),
            arr(&[
                span(0b110_00011..=0b110_00011),
                span(0b10_000000..=0b10_010110)
            ]),
        ])
    );
    assert_eq!(
        encode_range(0x80..=0x7FF),
        Ok(vec![arr(&[
            span(0b110_00010..=0b110_11111),
            span(0b10_000000..=0b10_111111)
        ])])
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
fn encode_entire_range() {
    let mut seq = Vec::<Sequence>::new();
    assert!(
        Utf8Coder
            .encode_entire_range(|spans| seq.push(Sequence::try_from(spans).unwrap()))
            .is_ok()
    );
    seq.sort();
    assert_eq!(Ok(seq.clone()), expect_range(0x0..=0x10FFFF));
    assert_eq!(Ok(seq), encode_range(0x0..=0x10FFFF));
}

#[test]
#[ignore]
fn bruteforce_entire_unicode_range() {
    let mut iteration = 0;
    let first_char = '\0';
    let last_char = '\u{10FFFF}';
    for start in first_char..=last_char {
        for end in start..=last_char {
            iteration += 1;
            assert_eq!(
                encode_range(start as u32..=end as u32),
                expect_range(start as u32..=end as u32),
                "range: [{start:X?}-{end:X?}], iteration: {iteration}",
            );
        }
    }
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
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn encode_ranges((start, end) in gen_range()) {
            assert_eq!(
                encode_range(start..=end),
                expect_range(start..=end),
            );
        }
    }
}
