use pretty_assertions::assert_eq;
use regex_syntax::utf8::Utf8Sequences;
use regr::{Span, span};
use renc::{Coder, Result, Utf8Coder};
use std::ops::RangeInclusive;

fn encode_range(range: RangeInclusive<u32>) -> Result<String> {
    let mut spans_seq = Vec::<Vec<Span>>::new();
    Utf8Coder.encode_range(range, |spans| spans_seq.push(Vec::from(spans)))?;
    spans_seq.sort();
    // spans_seq.sort_by(|a, b| format!("{:X?}", a).cmp(&format!("{:X?}", b)));
    Ok(format!("{:X?}", spans_seq))
}

fn _encode<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<String> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    encode_range(start..=end)
}

fn expect_range(range: RangeInclusive<u32>) -> Result<String> {
    let start = char::from_u32(*range.start()).unwrap();
    let end = char::from_u32(*range.end()).unwrap();
    let mut seq = Utf8Sequences::new(start, end).collect::<Vec<_>>();
    seq.sort();
    let output = format!("{:?}", seq);
    Ok(output.replace("][", ", "))
}

fn _expect<const N: usize>(start: &[u8; N], end: &[u8; N]) -> Result<String> {
    let start = str::from_utf8(start).unwrap().chars().next().unwrap() as u32;
    let end = str::from_utf8(end).unwrap().chars().next().unwrap() as u32;
    expect_range(start..=end)
}

fn expect_spans(sequences: &[Vec<Span>]) -> Result<String> {
    Ok(format!("{:X?}", sequences))
}

#[test]
fn encode_one_byte_ranges() {
    assert_eq!(encode_range(0..=0), expect_spans(&[vec![span(0..=0)]]));
    assert_eq!(encode_range(0..=23), expect_spans(&[vec![span(0..=23)]]));
    assert_eq!(
        encode_range(0..=0x7F),
        expect_spans(&[vec![span(0..=0x7F)]])
    );
}

#[test]
fn encode_two_byte_ranges() {
    assert_eq!(
        encode_range(0x80..=0x81),
        expect_spans(&[vec![
            span(0b110_00010..=0b110_00010),
            span(0b10_000000..=0b10_000001)
        ]])
    );
    assert_eq!(
        encode_range(0x83..=0x734),
        expect_spans(&[
            vec![
                span(0b110_00010..=0b110_00010),
                span(0b10_000011..=0b10_111111)
            ],
            vec![
                span(0b110_00011..=0b110_11011),
                span(0b10_000000..=0b10_111111)
            ],
            vec![
                span(0b110_11100..=0b110_11100),
                span(0b10_000000..=0b10_110100)
            ],
        ])
    );
    assert_eq!(
        encode_range(0x83..=0xD6),
        expect_spans(&[
            vec![
                span(0b110_00010..=0b110_00010),
                span(0b10_000011..=0b10_111111)
            ],
            vec![
                span(0b110_00011..=0b110_00011),
                span(0b10_000000..=0b10_010110)
            ],
        ])
    );
    assert_eq!(
        encode_range(0x80..=0x7FF),
        expect_spans(&[vec![
            span(0b110_00010..=0b110_11111),
            span(0b10_000000..=0b10_111111)
        ]])
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
        encode_range(0x10000..=0x10FFFF),
        expect_range(0x10000..=0x10FFFF)
    );
}

#[test]
fn encode_ranges() {
    assert_eq!(encode_range(0x0..=0x10FFFF), expect_range(0x0..=0x10FFFF));
}

#[test]
fn brutforce_encode_one_byte_ranges() {
    let mut iteration = 0;
    for start in '\u{0}'..='\u{7F}' {
        for end in start..='\u{7F}' {
            iteration += 1;
            let start = start as u32;
            let end = end as u32;
            assert_eq!(
                encode_range(start..=end),
                expect_range(start..=end),
                "range: {:X?}..={:X?}, iteration: {}",
                start,
                end,
                iteration
            );
        }
    }
}

#[test]
fn brutforce_encode_two_byte_ranges() {
    let mut iteration = 0;
    for start in '\u{80}'..='\u{7FF}' {
        for end in start..='\u{7FF}' {
            iteration += 1;
            let start = start as u32;
            let end = end as u32;
            assert_eq!(
                encode_range(start..=end),
                expect_range(start..=end),
                "range: {:X?}..={:X?}, iteration: {}",
                start,
                end,
                iteration
            );
        }
    }
}
