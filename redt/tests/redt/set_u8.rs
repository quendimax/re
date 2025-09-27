use pretty_assertions::assert_eq;
use redt::{RangeU8, SetU8};

#[test]
fn setu8_new() {
    let a = SetU8::new();
    let b = SetU8::default();
    assert_eq!(a, b);

    let a = SetU8::from(&[1, 2, 3]);
    let mut b = SetU8::from(2);
    b.merge_byte(1);
    b.merge_byte(3);
    assert_eq!(a, b);
}

#[test]
fn setu8_contains_byte() {
    let mut a = SetU8::new();
    a.merge_range((4..=30).into());
    assert!(a.contains_byte(30));
    assert!(!a.contains_byte(31));
}

#[test]
fn setu8_contains_range() {
    let mut a = SetU8::new();
    let r00 = RangeU8::new(10, 20);
    let r01 = RangeU8::new(10, 120);
    let r02 = RangeU8::new(10, 180);
    let r03 = RangeU8::new(10, 220);

    a.merge_range((4..=30).into());
    assert!(a.contains_range(r00));

    a.merge_range((31..=122).into());
    assert!(a.contains_range(r01));

    a.merge_range((123..=189).into());
    assert!(a.contains_range(r02));

    a.merge_range((190..=250).into());
    assert!(a.contains_range(r03));
}

#[test]
fn setu8_contains_set() {
    let mut a = SetU8::new();
    a.merge_range((0..=240).into());

    let mut b = SetU8::new();
    b.merge_byte(3);
    b.merge_byte(80);
    b.merge_byte(160);
    b.merge_byte(220);

    assert!(a.contains_set(&b));
}

#[test]
fn setu8_intersects_byte() {
    let mut a = SetU8::new();
    a.merge_range((4..=30).into());
    assert!(a.intersects_byte(30));
    assert!(!a.intersects_byte(31));
}

#[test]
fn setu8_intersects_range() {
    let mut a = SetU8::new();
    a.merge_range((198..=250).into());

    let r33 = RangeU8::new(210, 220);
    let r23 = RangeU8::new(180, 220);
    let r13 = RangeU8::new(100, 220);
    let r03 = RangeU8::new(20, 220);

    assert!(a.intersects_range(r33));
    assert!(a.intersects_range(r23));
    assert!(a.intersects_range(r13));
    assert!(a.intersects_range(r03));
}

#[test]
fn setu8_intersects_set() {
    let mut a = SetU8::new();
    a.merge_range((0..=240).into());

    let mut b = SetU8::new();
    b.merge_byte(220);

    assert!(a.intersects_set(&b));
}

#[test]
fn setu8_merge_byte() {
    let mut a = SetU8::new();
    a.merge_byte(3);
}

#[test]
fn setu8_merge_range() {
    let mut a = SetU8::new();
    a.merge_range((198..=250).into());
    a.merge_range((150..=250).into());
    a.merge_range((98..=250).into());
    a.merge_range((8..=250).into());
}

#[test]
fn setu8_merge_set() {
    let mut a = SetU8::new();
    a.merge_range((0..=240).into());

    let mut b = SetU8::new();
    b.merge_set(&a);

    assert_eq!(a, b);
}

#[test]
fn setu8_display_fmt() {
    let mut a = SetU8::new();
    a.merge_range((0..=200).into());
    a.merge_range((210..=240).into());

    assert_eq!(format!("{}", a), "[00h-C8h | D2h-F0h]");
}

#[test]
fn setu8_bytes() {
    let mut a = SetU8::new();
    a.merge_range((0..=240).into());

    assert_eq!(a.bytes().collect::<Vec<_>>(), (0..=240).collect::<Vec<_>>());
}

#[test]
fn setu8_ranges() {
    let mut a = SetU8::new();
    a.merge_range((0..=240).into());

    assert_eq!(
        a.ranges().collect::<Vec<_>>(),
        [0..=63, 64..=127, 128..=191, 192..=240]
            .iter()
            .map(|r| RangeU8::new(*r.start(), *r.end()))
            .collect::<Vec<_>>()
    );
}
