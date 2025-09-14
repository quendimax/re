use pretty_assertions::assert_eq;
use redt::{Legible, Range, range};

#[test]
fn range_new() {
    _ = Range::new(1, 2);
    _ = Range::new(2, 0);
    _ = Range::new_unchecked(0, 1);
    const _: Range<u8> = Range::new_unchecked_const(0, 1);
    assert_eq!(Range::new_checked(0, 0), Some(Range::from(0)));
    assert_eq!(Range::new_checked(0, 2), Some(Range::new(0, 2)));
    assert_eq!(Range::new_checked(3, 2), None);
}

#[test]
fn range_from_type() {
    let r = Range::from(3..=3);
    assert_eq!(r.start(), 3);
    assert_eq!(r.last(), 3);

    let r = Range::from(b'a'..=b'a');
    assert_eq!(r.start(), b'a');
    assert_eq!(r.last(), b'a');
}

#[test]
fn range_from_range_inclusive() {
    let r = Range::from(3..=5);
    assert_eq!(r.start(), 3);
    assert_eq!(r.last(), 5);

    #[allow(clippy::reversed_empty_ranges)]
    let r = Range::from(3..=0);
    assert_eq!(r.start(), 0);
    assert_eq!(r.last(), 3);
}

#[test]
fn range_fn() {
    let _: Range<u8> = range(3, 5);
    let _: Range<u8> = range(b'a', b'b');
    let _: Range<u64> = range(3u8, 5u32);
}

#[test]
fn range_width() {
    assert_eq!(range::<u8>(3, 4).width(), Some(2));
    assert_eq!(Range::new(7u16, 2).width(), Some(6));
    assert_eq!(Range::new(0u8, 254).width(), Some(255));
    assert_eq!(Range::new(0u8, 255).width(), None);
}

#[test]
fn range_set() {
    let mut sp = Range::new(2, 3);
    assert_eq!(sp.start(), 2);
    assert_eq!(sp.last(), 3);
    sp.set_start(3);
    sp.set_end(4);
    assert_eq!(sp.start(), 3);
    assert_eq!(sp.last(), 4);
}

#[test]
#[should_panic]
fn range_set_start_panics() {
    let mut sp = Range::new(2, 3);
    sp.set_start(4);
}

#[test]
#[should_panic]
fn range_set_end_panics() {
    let mut sp = Range::new(2, 3);
    sp.set_end(1);
}

#[test]
fn range_is_at_left() {
    let r_0_1 = Range::from(b'0'..=b'1');
    let r_2_4 = Range::from(b'2'..=b'4');
    let r_3_5 = Range::from(b'3'..=b'5');

    assert!(r_0_1.is_at_left(&r_2_4));
    assert!(r_0_1.is_at_left(&r_3_5));
    assert!(!r_2_4.is_at_left(&r_3_5));

    assert!(!r_2_4.is_at_left(&r_0_1));
    assert!(!r_3_5.is_at_left(&r_0_1));
    assert!(!r_3_5.is_at_left(&r_2_4));
}

#[test]
fn range_is_at_right() {
    let r_0_1 = Range::from(b'0'..=b'1');
    let r_2_4 = Range::from(b'2'..=b'4');
    let r_3_5 = Range::from(b'3'..=b'5');

    assert!(r_2_4.is_at_right(&r_0_1));
    assert!(r_3_5.is_at_right(&r_0_1));
    assert!(!r_3_5.is_at_right(&r_2_4));

    assert!(!r_0_1.is_at_right(&r_2_4));
    assert!(!r_0_1.is_at_right(&r_3_5));
    assert!(!r_2_4.is_at_right(&r_3_5));
}

#[test]
fn range_intersects() {
    let r_0 = Range::new(b'0', b'0');
    let r_1_4 = Range::from(b'1'..=b'4');
    let r_2_3 = Range::from(b'2'..=b'3');
    let r_4_5 = Range::from(b'4'..=b'5');

    assert!(!r_0.intersects(&r_1_4));
    assert!(r_1_4.intersects(&r_2_3));
    assert!(r_1_4.intersects(&r_4_5));
    assert!(!r_2_3.intersects(&r_4_5));

    // reverted
    assert!(!r_1_4.intersects(&r_0));
    assert!(r_2_3.intersects(&r_1_4));
    assert!(r_4_5.intersects(&r_1_4));
    assert!(!r_4_5.intersects(&r_2_3));
}

#[test]
fn range_contains() {
    let r_0 = Range::new(b'0', b'0');
    let r_1_4 = Range::from(b'1'..=b'4');
    let r_2_3 = Range::from(b'2'..=b'3');
    let r_4_5 = Range::from(b'4'..=b'5');

    assert!(!r_0.contains(&r_1_4));
    assert!(r_1_4.contains(&r_2_3));
    assert!(!r_1_4.contains(&r_4_5));
    assert!(!r_2_3.contains(&r_4_5));

    // reverted
    assert!(!r_1_4.contains(&r_0));
    assert!(!r_2_3.contains(&r_1_4));
    assert!(!r_4_5.contains(&r_1_4));
    assert!(!r_4_5.contains(&r_2_3));
}

#[test]
fn range_adjoins() {
    let r_0_1 = Range::from(b'0'..=b'1');
    let r_2_4 = Range::from(b'2'..=b'4');
    let r_3_5 = Range::from(b'3'..=b'5');
    let r_5_7 = Range::from(b'5'..=b'7');
    let r_6 = Range::new(b'6', b'6');

    assert!(r_0_1.adjoins(&r_2_4));
    assert!(!r_0_1.adjoins(&r_3_5));
    assert!(!r_2_4.adjoins(&r_3_5));
    assert!(!r_3_5.adjoins(&r_5_7));
    assert!(!r_5_7.adjoins(&r_6));

    // reverted
    assert!(r_2_4.adjoins(&r_0_1));
    assert!(!r_3_5.adjoins(&r_0_1));
    assert!(!r_3_5.adjoins(&r_2_4));
    assert!(!r_5_7.adjoins(&r_3_5));
    assert!(!r_6.adjoins(&r_5_7));
}

#[test]
fn range_try_merge() {
    let r_0_1 = Range::from(b'0'..=b'1');
    let r_2_4 = Range::from(b'2'..=b'4');
    let r_3_5 = Range::from(b'3'..=b'5');
    let r_5_7 = Range::from(b'5'..=b'7');
    let r_6 = Range::new(b'6', b'6');

    assert_eq!(r_0_1.try_merge(&r_2_4), Some((b'0'..=b'4').into()));
    assert_eq!(r_2_4.try_merge(&r_3_5), Some((b'2'..=b'5').into()));
    assert_eq!(r_3_5.try_merge(&r_5_7), Some((b'3'..=b'7').into()));
    assert_eq!(r_5_7.try_merge(&r_6), Some(r_5_7));

    // reverted
    assert_eq!(r_2_4.try_merge(&r_0_1), Some((b'0'..=b'4').into()));
    assert_eq!(r_3_5.try_merge(&r_2_4), Some((b'2'..=b'5').into()));
    assert_eq!(r_5_7.try_merge(&r_3_5), Some((b'3'..=b'7').into()));
    assert_eq!(r_6.try_merge(&r_5_7), Some(r_5_7));

    assert_eq!(r_0_1.try_merge(&r_5_7), None);
    assert_eq!(r_5_7.try_merge(&r_0_1), None);
}

#[test]
#[should_panic]
fn range_merge_panic() {
    let r_0_1 = Range::from(b'0'..=b'1');
    let r_5_7 = Range::from(b'5'..=b'7');
    r_0_1.merge(&r_5_7);
}

#[test]
fn symbol_range_debug_fmt() {
    assert_eq!(format!("{:?}", Range::from(b'a'..=b'a')), r"97");
    assert_eq!(format!("{:?}", Range::from(b'\0'..=b'Z')), r"0-90");
    assert_eq!(format!("{:?}", Range::from(b'\x7E'..=b'~')), r"126");
}

#[test]
fn symbol_range_binary_fmt() {
    assert_eq!(format!("{:b}", range::<u8>(b'a', b'a')), r"1100001");
    assert_eq!(format!("{:b}", range::<u8>(b'\0', b'Z')), r"0-1011010");
    assert_eq!(format!("{:b}", range::<u8>(b'\x7E', b'~')), r"1111110");
}

#[test]
fn symbol_range_octal_fmt() {
    assert_eq!(format!("{:o}", range::<u8>(b'a', b'a')), r"141");
    assert_eq!(format!("{:o}", range::<u8>(b'\0', b'Z')), r"0-132");
    assert_eq!(format!("{:o}", range::<u8>(b'\x7E', b'~')), r"176");
}

#[test]
fn symbol_range_lowerhex_fmt() {
    assert_eq!(format!("{:x}", range::<u8>(b'\0', b'Z')), r"0-5a");
    assert_eq!(format!("{:x}", range::<u8>(b'\x7E', b'~')), r"7e");
}

#[test]
fn symbol_range_upperhex_fmt() {
    assert_eq!(format!("{:X}", range::<u8>(b'\0', b'Z')), r"0-5A");
    assert_eq!(format!("{:X}", range::<u8>(b'\x7E', b'~')), r"7E");
}

#[test]
fn symbol_range_legible_display() {
    assert_eq!(format!("{}", Range::from(b'a'..=b'a').display()), r"'a'");
    assert_eq!(
        format!("{}", Range::from(b'\0'..=b'Z').display()),
        r"00h-'Z'"
    );
    assert_eq!(format!("{}", Range::from(b'\x7E'..=b'~').display()), r"'~'");

    assert_eq!(format!("{}", Range::from(b'a'..=b'a')), r"'a'");
    assert_eq!(format!("{}", Range::from(b'\0'..=b'Z')), r"00h-'Z'");
    assert_eq!(format!("{}", Range::from(b'\x7E'..=b'~')), r"'~'");
}
