use pretty_assertions::assert_eq;
use redt::{Range, RangeList};

#[test]
fn range_list_ctor() {
    assert!(RangeList::<u32>::default().is_empty());
    let r = RangeList::<u32>::new(1, 5);
    assert_eq!(r.is_empty(), false);
    assert_eq!(r.len(), 1);
    assert_eq!(RangeList::<u32>::from(Range::from(0)).len(), 1);

    let r = RangeList::<u32>::from([Range::<u32>::new(1, 5), Range::<u32>::new(7, 9)]);
    assert_eq!(r.len(), 2);

    let r = RangeList::<u32>::from(&[Range::new(1, 5), Range::new(6, 9)]);
    assert_eq!(r.len(), 1);
}

#[test]
fn range_list_merge() {
    let mut list = RangeList::<u32>::default();
    assert!(list.is_empty());

    list.merge(Range::new(3, 5));
    assert_eq!(list.len(), 1);

    list.merge(Range::new(6, 9));
    assert_eq!(list.ranges(), &[Range::new(3, 9)]);

    list.merge(Range::new(11, 12));
    assert_eq!(list.ranges(), &[Range::new(3, 9), Range::new(11, 12)]);

    list.merge(Range::from(10));
    assert_eq!(list.ranges(), &[Range::new(3, 12)]);

    list.merge(Range::new(16, 17));
    assert_eq!(list.ranges(), &[Range::new(3, 12), Range::new(16, 17)]);

    list.merge(Range::from(15));
    assert_eq!(list.ranges(), &[Range::new(3, 12), Range::new(15, 17)]);

    list.merge(Range::from(1));
    assert_eq!(
        list.ranges(),
        &[Range::from(1), Range::new(3, 12), Range::new(15, 17)]
    );

    list.merge(Range::new(1, 16));
    assert_eq!(list.ranges(), &[Range::new(1, 17)]);

    list.merge(Range::new(0, 16));
    assert_eq!(list.ranges(), &[Range::new(0, 17)]);

    list.merge(Range::from(21));
    list.merge(Range::from(19));
    assert_eq!(
        list.ranges(),
        &[Range::new(0, 17), Range::from(19), Range::from(21)]
    );

    list.merge(Range::from(10));
    assert_eq!(
        list.ranges(),
        &[Range::new(0, 17), Range::from(19), Range::from(21)]
    );
}

#[test]
fn range_list_exclude() {
    // empty list
    let mut list = RangeList::<u32>::default();
    list.exclude(Range::new(3, 5));
    assert_eq!(list.ranges(), &[]);

    let mut list = RangeList::<u32>::default();
    list.merge(Range::new(0, 14));
    list.exclude(Range::new(5, 8));
    assert_eq!(list.ranges(), &[Range::new(0, 4), Range::new(9, 14)]);

    list.exclude(Range::from(0));
    assert_eq!(list.ranges(), &[Range::new(1, 4), Range::new(9, 14)]);

    list.exclude(Range::new(0, 1));
    assert_eq!(list.ranges(), &[Range::new(2, 4), Range::new(9, 14)]);

    list.exclude(Range::from(0));
    assert_eq!(list.ranges(), &[Range::new(2, 4), Range::new(9, 14)]);

    list.exclude(Range::new(4, 9));
    assert_eq!(list.ranges(), &[Range::new(2, 3), Range::new(10, 14)]);

    list.exclude(Range::new(14, 19));
    assert_eq!(list.ranges(), &[Range::new(2, 3), Range::new(10, 13)]);

    list.exclude(Range::new(19, 20));
    assert_eq!(list.ranges(), &[Range::new(2, 3), Range::new(10, 13)]);

    list.merge(Range::from(0));
    list.merge(Range::from(5));
    list.merge(Range::from(7));
    list.exclude(Range::new(1, 10));
    assert_eq!(list.ranges(), &[0.into(), Range::new(11, 13)]);

    list.exclude(Range::new(12, 13));
    assert_eq!(list.ranges(), &[0.into(), Range::new(11, 11)]);

    let mut list = RangeList::<u32>::default();
    list.merge(Range::new(1, 20));

    list.exclude(Range::new(1, 2));
    assert_eq!(list.ranges(), &[Range::new(3, 20)]);

    list.exclude(Range::new(1, 4));
    assert_eq!(list.ranges(), &[Range::new(5, 20)]);

    list.exclude(Range::new(19, 20));
    assert_eq!(list.ranges(), &[Range::new(5, 18)]);

    list.exclude(Range::new(17, 20));
    assert_eq!(list.ranges(), &[Range::new(5, 16)]);

    list.exclude(Range::new(5, 16));
    assert_eq!(list.ranges(), &[]);
}

#[test]
fn range_list_fmt() {
    let mut list = RangeList::<u8>::default();
    list.merge(Range::new(3, 10));
    list.merge(Range::new(13, 13));
    list.merge(Range::new(61, u8::MAX));

    assert_eq!(format!("{list}"), "03h-0Ah | 0Dh | '='-FFh");
    assert_eq!(format!("{list:?}"), "3-10 | 13 | 61-255");
    assert_eq!(format!("{list:b}"), "11-1010 | 1101 | 111101-11111111");
    assert_eq!(format!("{list:o}"), "3-12 | 15 | 75-377");
    assert_eq!(format!("{list:x}"), "3-a | d | 3d-ff");
    assert_eq!(format!("{list:X}"), "3-A | D | 3D-FF");
}
