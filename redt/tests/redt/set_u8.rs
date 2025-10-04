use pretty_assertions::assert_eq;
use redt::ops::*;
use redt::{RangeU8, SetU8};

#[test]
fn setu8_new() {
    let a = SetU8::new();
    let b = SetU8::default();
    assert_eq!(a, b);

    let a = SetU8::from(&[1, 2, 3]);
    let mut b = SetU8::from(2);
    b.include(1);
    b.include(3);
    assert_eq!(a, b);
}

#[test]
fn setu8_contains_byte() {
    let a = SetU8::from(4..=30);
    assert!(a.contains(30));
    assert!(!a.contains(31));
}

#[test]
fn setu8_contains_range() {
    let mut a = SetU8::new();
    let r00 = RangeU8::new(10, 20);
    let r01 = RangeU8::new(10, 120);
    let r02 = RangeU8::new(10, 180);
    let r03 = RangeU8::new(10, 220);

    a.include(4..=30);
    assert!(a.contains(r00));

    a.include(31..=122);
    assert!(a.contains(r01));

    a.include(123..=189);
    assert!(a.contains(r02));

    a.include(190..=250);
    assert!(a.contains(r03));
}

#[test]
fn setu8_contains_set() {
    let mut a = SetU8::new();
    a.include(0..=240);

    let mut b = SetU8::new();
    b.include(3);
    b.include(80);
    b.include(160);
    b.include(220);

    assert!(a.contains(&b));
}

#[test]
fn setu8_intersects_byte() {
    let mut a = SetU8::new();
    a.include(4..=30);
    assert!(a.intersects(30));
    assert!(!a.intersects(31));
}

#[test]
fn setu8_intersects_range() {
    let mut a = SetU8::new();
    a.include(198..=250);

    let r33 = RangeU8::new(210, 220);
    let r23 = RangeU8::new(180, 220);
    let r13 = RangeU8::new(100, 220);
    let r03 = RangeU8::new(20, 220);

    assert!(a.intersects(r33));
    assert!(a.intersects(r23));
    assert!(a.intersects(r13));
    assert!(a.intersects(r03));
}

#[test]
fn setu8_intersects_set() {
    let a = SetU8::from(0..=240);
    let b = SetU8::from(220);

    assert!(a.intersects(&b));
}

#[test]
fn setu8_merge_byte() {
    let mut a = SetU8::new();
    a.include(3);
}

#[test]
fn setu8_merge_range() {
    let mut a = SetU8::new();
    a.include(198..=250);
    a.include(150..=250);
    a.include(98..=250);
    a.include(8..=250);
}

#[test]
fn setu8_merge_set() {
    let a = SetU8::from(0..=240);
    let mut b = SetU8::new();
    b.include(&a);

    assert_eq!(a, b);
}

#[test]
fn setu8_display_fmt() {
    let mut a = SetU8::new();
    a.include(0..=200);
    a.include(210..=240);

    assert_eq!(format!("{}", a), "[00h-C8h | D2h-F0h]");
}

#[test]
fn setu8_bytes() {
    let a = SetU8::from(0..=240);
    assert_eq!(a.bytes().collect::<Vec<_>>(), (0..=240).collect::<Vec<_>>());
}

#[test]
fn setu8_ranges() {
    let mut a = SetU8::new();
    a.include(RangeU8::from(0..=240));

    assert_eq!(
        a.ranges().collect::<Vec<_>>(),
        [0..=63, 64..=127, 128..=191, 192..=240]
            .iter()
            .map(|r| RangeU8::new(*r.start(), *r.end()))
            .collect::<Vec<_>>()
    );
}
