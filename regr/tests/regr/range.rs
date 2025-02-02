use pretty_assertions::assert_eq;
use regr::{err, range, Range};

#[test]
fn range_from_type() {
    let r = Range::from(3);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 3);

    let r = Range::from('a');
    assert_eq!(r.start(), 'a');
    assert_eq!(r.end(), 'a');
}

#[test]
fn range_from_range_inclusive() {
    let r = Range::from(3..=5);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 5);

    let r = Range::from(3..=3);
    assert_eq!(r.start(), 3);
    assert_eq!(r.end(), 3);

    let r = Range::from(3..=0);
    assert_eq!(r.start(), 0);
    assert_eq!(r.end(), 3);
}

#[test]
fn range_fn() {
    let _: Range<i32> = range(3..=3);
    let _: Range<u32> = range(3..=3);
    let _ = range(3);
    let _: Range<u32> = range(3);
    let _ = range::<char>('a');
}

#[test]
fn range_is_at_left() {
    let r_0_1 = Range::from('0'..='1');
    let r_2_4 = Range::from('2'..='4');
    let r_3_5 = Range::from('3'..='5');

    assert!(r_0_1.is_at_left(&r_2_4));
    assert!(r_0_1.is_at_left(&r_3_5));
    assert!(!r_2_4.is_at_left(&r_3_5));

    assert!(!r_2_4.is_at_left(&r_0_1));
    assert!(!r_3_5.is_at_left(&r_0_1));
    assert!(!r_3_5.is_at_left(&r_2_4));
}

#[test]
fn range_is_at_right() {
    let r_0_1 = Range::from('0'..='1');
    let r_2_4 = Range::from('2'..='4');
    let r_3_5 = Range::from('3'..='5');

    assert!(r_2_4.is_at_right(&r_0_1));
    assert!(r_3_5.is_at_right(&r_0_1));
    assert!(!r_3_5.is_at_right(&r_2_4));

    assert!(!r_0_1.is_at_right(&r_2_4));
    assert!(!r_0_1.is_at_right(&r_3_5));
    assert!(!r_2_4.is_at_right(&r_3_5));
}

#[test]
fn range_intersects() {
    let r_0 = Range::from('0');
    let r_1_4 = Range::from('1'..='4');
    let r_2_3 = Range::from('2'..='3');
    let r_4_5 = Range::from('4'..='5');

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
fn range_adjoins() {
    let r_0_1 = Range::from('0'..='1');
    let r_2_4 = Range::from('2'..='4');
    let r_3_5 = Range::from('3'..='5');
    let r_5_7 = Range::from('5'..='7');
    let r_6 = Range::from('6');

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
    let r_0_1 = Range::from('0'..='1');
    let r_2_4 = Range::from('2'..='4');
    let r_3_5 = Range::from('3'..='5');
    let r_5_7 = Range::from('5'..='7');
    let r_6 = Range::from('6');

    assert_eq!(r_0_1.try_merge(&r_2_4), Ok(('0'..='4').into()));
    assert_eq!(r_2_4.try_merge(&r_3_5), Ok(('2'..='5').into()));
    assert_eq!(r_3_5.try_merge(&r_5_7), Ok(('3'..='7').into()));
    assert_eq!(r_5_7.try_merge(&r_6), Ok(r_5_7.clone()));

    // reverted
    assert_eq!(r_2_4.try_merge(&r_0_1), Ok(('0'..='4').into()));
    assert_eq!(r_3_5.try_merge(&r_2_4), Ok(('2'..='5').into()));
    assert_eq!(r_5_7.try_merge(&r_3_5), Ok(('3'..='7').into()));
    assert_eq!(r_6.try_merge(&r_5_7), Ok(r_5_7.clone()));

    assert_eq!(r_0_1.try_merge(&r_5_7), err::merge_delimited_ranges());
    assert_eq!(r_5_7.try_merge(&r_0_1), err::merge_delimited_ranges());
}

#[test]
#[should_panic]
fn range_merge_panic() {
    let r_0_1 = Range::from('0'..='1');
    let r_5_7 = Range::from('5'..='7');
    r_0_1.merge(&r_5_7);
}

#[test]
fn symbol_range_debug_fmt() {
    assert_eq!(format!("{:?}", Range::from('a'..='a')), r"['a']");
    assert_eq!(format!("{:?}", Range::from('\0'..='ў')), r"['\0'-'ў']");
    assert_eq!(
        format!("{:?}", Range::from('\u{9F}'..='ў')),
        r"['\u{9f}'-'ў']"
    );
}
