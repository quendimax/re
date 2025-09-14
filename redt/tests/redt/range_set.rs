use pretty_assertions::assert_eq;
use redt::{Range, RangeSet};

#[test]
fn range_set_ctor() {
    assert!(RangeSet::<u32>::default().is_empty());
    let r = RangeSet::<u32>::new(1, 5);
    assert_eq!(r.is_empty(), false);
    assert_eq!(r.len(), 1);
    assert_eq!(RangeSet::<u32>::from(Range::from(0)).len(), 1);

    let r = RangeSet::<u32>::from([Range::<u32>::new(1, 5), Range::<u32>::new(7, 9)]);
    assert_eq!(r.len(), 2);

    let r = RangeSet::<u32>::from(&[Range::new(1, 5), Range::new(6, 9)]);
    assert_eq!(r.len(), 1);
}

#[test]
fn range_set_merge() {
    let mut set = RangeSet::<u32>::default();
    assert!(set.is_empty());

    set.merge(Range::new(3, 5));
    assert_eq!(set.len(), 1);

    set.merge(Range::new(6, 9));
    assert_eq!(set.ranges(), &[Range::new(3, 9)]);

    set.merge(Range::new(11, 12));
    assert_eq!(set.ranges(), &[Range::new(3, 9), Range::new(11, 12)]);

    set.merge(Range::from(10));
    assert_eq!(set.ranges(), &[Range::new(3, 12)]);

    set.merge(Range::new(16, 17));
    assert_eq!(set.ranges(), &[Range::new(3, 12), Range::new(16, 17)]);

    set.merge(Range::from(15));
    assert_eq!(set.ranges(), &[Range::new(3, 12), Range::new(15, 17)]);

    set.merge(Range::from(1));
    assert_eq!(
        set.ranges(),
        &[Range::from(1), Range::new(3, 12), Range::new(15, 17)]
    );

    set.merge(Range::new(1, 16));
    assert_eq!(set.ranges(), &[Range::new(1, 17)]);

    set.merge(Range::new(0, 16));
    assert_eq!(set.ranges(), &[Range::new(0, 17)]);

    set.merge(Range::from(21));
    set.merge(Range::from(19));
    assert_eq!(
        set.ranges(),
        &[Range::new(0, 17), Range::from(19), Range::from(21)]
    );

    set.merge(Range::from(10));
    assert_eq!(
        set.ranges(),
        &[Range::new(0, 17), Range::from(19), Range::from(21)]
    );
}

#[test]
fn range_set_exclude() {
    // empty set
    let mut set = RangeSet::<u32>::default();
    set.exclude(Range::new(3, 5));
    assert_eq!(set.ranges(), &[]);

    let mut set = RangeSet::<u32>::default();
    set.merge(Range::new(0, 14));
    set.exclude(Range::new(5, 8));
    assert_eq!(set.ranges(), &[Range::new(0, 4), Range::new(9, 14)]);

    set.exclude(Range::from(0));
    assert_eq!(set.ranges(), &[Range::new(1, 4), Range::new(9, 14)]);

    set.exclude(Range::new(0, 1));
    assert_eq!(set.ranges(), &[Range::new(2, 4), Range::new(9, 14)]);

    set.exclude(Range::from(0));
    assert_eq!(set.ranges(), &[Range::new(2, 4), Range::new(9, 14)]);

    set.exclude(Range::new(4, 9));
    assert_eq!(set.ranges(), &[Range::new(2, 3), Range::new(10, 14)]);

    set.exclude(Range::new(14, 19));
    assert_eq!(set.ranges(), &[Range::new(2, 3), Range::new(10, 13)]);

    set.exclude(Range::new(19, 20));
    assert_eq!(set.ranges(), &[Range::new(2, 3), Range::new(10, 13)]);

    set.merge(Range::from(0));
    set.merge(Range::from(5));
    set.merge(Range::from(7));
    set.exclude(Range::new(1, 10));
    assert_eq!(set.ranges(), &[0.into(), Range::new(11, 13)]);

    set.exclude(Range::new(12, 13));
    assert_eq!(set.ranges(), &[0.into(), Range::new(11, 11)]);

    let mut set = RangeSet::<u32>::default();
    set.merge(Range::new(1, 20));

    set.exclude(Range::new(1, 2));
    assert_eq!(set.ranges(), &[Range::new(3, 20)]);

    set.exclude(Range::new(1, 4));
    assert_eq!(set.ranges(), &[Range::new(5, 20)]);

    set.exclude(Range::new(19, 20));
    assert_eq!(set.ranges(), &[Range::new(5, 18)]);

    set.exclude(Range::new(17, 20));
    assert_eq!(set.ranges(), &[Range::new(5, 16)]);

    set.exclude(Range::new(5, 16));
    assert_eq!(set.ranges(), &[]);
}

#[test]
fn range_set_fmt() {
    let mut set = RangeSet::<u8>::default();
    set.merge(Range::new(3, 10));
    set.merge(Range::new(13, 13));
    set.merge(Range::new(61, u8::MAX));

    assert_eq!(format!("{set}"), "03h-0Ah | 0Dh | '='-FFh");
    assert_eq!(format!("{set:?}"), "3-10 | 13 | 61-255");
    assert_eq!(format!("{set:b}"), "11-1010 | 1101 | 111101-11111111");
    assert_eq!(format!("{set:o}"), "3-12 | 15 | 75-377");
    assert_eq!(format!("{set:x}"), "3-a | d | 3d-ff");
    assert_eq!(format!("{set:X}"), "3-A | D | 3D-FF");
}
