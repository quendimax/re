use pretty_assertions::assert_eq;
use regr::{Span, span};

#[test]
fn span_from_type() {
    let r = Span::from(3);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 3);

    let r = Span::from(b'a');
    assert_eq!(r.start(), b'a');
    assert_eq!(r.end(), b'a');
}

#[test]
fn span_from_span_inclusive() {
    let r = Span::from(3..=5);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 5);

    let r = Span::from(3..=3);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 3);

    #[allow(clippy::reversed_empty_ranges)]
    let r = Span::from(3..=0);
    assert_eq!(r.start(), 0);
    assert_eq!(r.end(), 3);
}

#[test]
fn span_fn() {
    let _ = span(3);
    let _ = span(b'a');
}

#[test]
fn span_is_at_left() {
    let r_0_1 = Span::from(b'0'..=b'1');
    let r_2_4 = Span::from(b'2'..=b'4');
    let r_3_5 = Span::from(b'3'..=b'5');

    assert!(r_0_1.is_at_left(r_2_4));
    assert!(r_0_1.is_at_left(r_3_5));
    assert!(!r_2_4.is_at_left(r_3_5));

    assert!(!r_2_4.is_at_left(r_0_1));
    assert!(!r_3_5.is_at_left(r_0_1));
    assert!(!r_3_5.is_at_left(r_2_4));
}

#[test]
fn span_is_at_right() {
    let r_0_1 = Span::from(b'0'..=b'1');
    let r_2_4 = Span::from(b'2'..=b'4');
    let r_3_5 = Span::from(b'3'..=b'5');

    assert!(r_2_4.is_at_right(r_0_1));
    assert!(r_3_5.is_at_right(r_0_1));
    assert!(!r_3_5.is_at_right(r_2_4));

    assert!(!r_0_1.is_at_right(r_2_4));
    assert!(!r_0_1.is_at_right(r_3_5));
    assert!(!r_2_4.is_at_right(r_3_5));
}

#[test]
fn span_intersects() {
    let r_0 = Span::from(b'0');
    let r_1_4 = Span::from(b'1'..=b'4');
    let r_2_3 = Span::from(b'2'..=b'3');
    let r_4_5 = Span::from(b'4'..=b'5');

    assert!(!r_0.intersects(r_1_4));
    assert!(r_1_4.intersects(r_2_3));
    assert!(r_1_4.intersects(r_4_5));
    assert!(!r_2_3.intersects(r_4_5));

    // reverted
    assert!(!r_1_4.intersects(r_0));
    assert!(r_2_3.intersects(r_1_4));
    assert!(r_4_5.intersects(r_1_4));
    assert!(!r_4_5.intersects(r_2_3));
}

#[test]
fn span_adjoins() {
    let r_0_1 = Span::from(b'0'..=b'1');
    let r_2_4 = Span::from(b'2'..=b'4');
    let r_3_5 = Span::from(b'3'..=b'5');
    let r_5_7 = Span::from(b'5'..=b'7');
    let r_6 = Span::from(b'6');

    assert!(r_0_1.adjoins(r_2_4));
    assert!(!r_0_1.adjoins(r_3_5));
    assert!(!r_2_4.adjoins(r_3_5));
    assert!(!r_3_5.adjoins(r_5_7));
    assert!(!r_5_7.adjoins(r_6));

    // reverted
    assert!(r_2_4.adjoins(r_0_1));
    assert!(!r_3_5.adjoins(r_0_1));
    assert!(!r_3_5.adjoins(r_2_4));
    assert!(!r_5_7.adjoins(r_3_5));
    assert!(!r_6.adjoins(r_5_7));
}

#[test]
fn span_try_merge() {
    let r_0_1 = Span::from(b'0'..=b'1');
    let r_2_4 = Span::from(b'2'..=b'4');
    let r_3_5 = Span::from(b'3'..=b'5');
    let r_5_7 = Span::from(b'5'..=b'7');
    let r_6 = Span::from(b'6');

    assert_eq!(r_0_1.try_merge(r_2_4), Some((b'0'..=b'4').into()));
    assert_eq!(r_2_4.try_merge(r_3_5), Some((b'2'..=b'5').into()));
    assert_eq!(r_3_5.try_merge(r_5_7), Some((b'3'..=b'7').into()));
    assert_eq!(r_5_7.try_merge(r_6), Some(r_5_7));

    // reverted
    assert_eq!(r_2_4.try_merge(r_0_1), Some((b'0'..=b'4').into()));
    assert_eq!(r_3_5.try_merge(r_2_4), Some((b'2'..=b'5').into()));
    assert_eq!(r_5_7.try_merge(r_3_5), Some((b'3'..=b'7').into()));
    assert_eq!(r_6.try_merge(r_5_7), Some(r_5_7));

    assert_eq!(r_0_1.try_merge(r_5_7), None);
    assert_eq!(r_5_7.try_merge(r_0_1), None);
}

#[test]
#[should_panic]
fn span_merge_panic() {
    let r_0_1 = Span::from(b'0'..=b'1');
    let r_5_7 = Span::from(b'5'..=b'7');
    r_0_1.merge(r_5_7);
}

#[test]
fn symbol_span_display_fmt() {
    assert_eq!(format!("{}", Span::from(b'a'..=b'a')), r"'a'");
    assert_eq!(format!("{}", Span::from(b'\0'..=b'Z')), r"00h-'Z'");
    assert_eq!(format!("{}", Span::from(b'\x7E'..=b'~')), r"'~'");
}

#[test]
fn symbol_span_debug_fmt() {
    assert_eq!(format!("{:?}", Span::from(b'a'..=b'a')), r"97");
    assert_eq!(format!("{:?}", Span::from(b'\0'..=b'Z')), r"0-90");
    assert_eq!(format!("{:?}", Span::from(b'\x7E'..=b'~')), r"126");
}

#[test]
fn symbol_span_binary_fmt() {
    assert_eq!(format!("{:b}", span(b'a'..=b'a')), r"1100001");
    assert_eq!(format!("{:b}", span(b'\0'..=b'Z')), r"0-1011010");
    assert_eq!(format!("{:b}", span(b'\x7E'..=b'~')), r"1111110");
}

#[test]
fn symbol_span_octal_fmt() {
    assert_eq!(format!("{:o}", span(b'a'..=b'a')), r"141");
    assert_eq!(format!("{:o}", span(b'\0'..=b'Z')), r"0-132");
    assert_eq!(format!("{:o}", span(b'\x7E'..=b'~')), r"176");
}

#[test]
fn symbol_span_lowerhex_fmt() {
    assert_eq!(format!("{:x}", span(b'\0'..=b'Z')), r"0-5a");
    assert_eq!(format!("{:x}", span(b'\x7E'..=b'~')), r"7e");
}

#[test]
fn symbol_span_upperhex_fmt() {
    assert_eq!(format!("{:X}", span(b'\0'..=b'Z')), r"0-5A");
    assert_eq!(format!("{:X}", span(b'\x7E'..=b'~')), r"7E");
}
