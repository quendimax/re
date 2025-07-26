use pretty_assertions::assert_eq;
use redt::RangeU8;
use regr::{Arena, Epsilon, Transition};

fn brange(range: impl Into<RangeU8>) -> RangeU8 {
    range.into()
}

#[test]
fn transition_new() {
    let arena = Arena::new();
    let transition = Transition::new_in(&arena);
    assert_eq!(transition.symbols().collect::<Vec<_>>(), vec![]);
}

#[test]
fn transition_from_chunks() {
    let arena = Arena::new();
    let transition = Transition::from_chunks_in(&[0, 1, 2, 3], &arena);
    assert_eq!(
        transition.symbols().collect::<Vec<_>>(),
        vec![64, 129, 192, 193]
    );
}

#[test]
fn transition_from_symbols() {
    let arena = Arena::new();
    let transition = Transition::from_symbols_in(b"\0abc\xFF", &arena);
    assert_eq!(
        transition.symbols().collect::<Vec<_>>(),
        vec![0, 97, 98, 99, 255]
    );
}

#[test]
fn transition_from_symbol() {
    let arena = Arena::new();
    let tr = Transition::from_symbol_in(8, &arena);
    assert!(tr.contains(8));
}

#[test]
fn transition_symbols() {
    type Vec = smallvec::SmallVec<[u8; 8]>;
    fn symbols(a: u64, b: u64, c: u64, d: u64) -> Vec {
        let arena = Arena::new();
        let smap = Transition::from_chunks_in(&[a, b, c, d], &arena);
        smap.symbols().collect::<Vec>()
    }
    fn vec<const N: usize>(buf: [u8; N]) -> Vec {
        Vec::from(&buf as &[u8])
    }

    assert_eq!(symbols(0, 0, 0, 0), vec([]));
    assert_eq!(symbols(255, 0, 0, 0), (0..=7).collect::<Vec>());
    assert_eq!(symbols(u64::MAX, 0, 0, 0), (0..=63).collect::<Vec>());
    assert_eq!(symbols(u64::MAX, 255, 0, 0), (0..=71).collect::<Vec>());
    assert_eq!(symbols(0x8000000000000001, 1, 0, 0), vec([0, 63, 64]));
    assert_eq!(symbols(0x5555, 0, 0, 0), vec([0, 2, 4, 6, 8, 10, 12, 14]));
    assert_eq!(
        symbols(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
        (0..=255).collect::<Vec>()
    );

    let arena = Arena::new();
    let tr = Transition::epsilon_in(&arena);
    assert_eq!(tr.symbols().next(), None);
}

#[test]
fn transition_ranges() {
    type Vec = smallvec::SmallVec<[RangeU8; 4]>;
    fn ranges(a: u64, b: u64, c: u64, d: u64) -> Vec {
        let arena = Arena::new();
        let smap = Transition::from_chunks_in(&[a, b, c, d], &arena);
        smap.ranges().collect::<Vec>()
    }
    fn vec<const N: usize>(buf: [RangeU8; N]) -> Vec {
        Vec::from(&buf as &[RangeU8])
    }

    assert_eq!(ranges(0, 0, 0, 0), vec([]));
    assert_eq!(ranges(255, 0, 0, 0), vec([brange(0..=7)]));
    assert_eq!(
        ranges(255, 255, 0, 0),
        vec([brange(0..=7), brange(64..=71)])
    );
    assert_eq!(ranges(0, 255, 0, 0), vec([brange(64..=71)]));
    assert_eq!(ranges(0, 0, 0, 255), vec([brange(192..=199)]));
    assert_eq!(
        ranges(255, 255, 255, 255),
        vec([
            brange(0..=7),
            brange(64..=71),
            brange(128..=135),
            brange(192..=199)
        ])
    );
    assert_eq!(ranges(u64::MAX, 0, 0, 0), vec([brange(0..=63)]));
    assert_eq!(ranges(0, u64::MAX, 0, 0), vec([brange(64..=127)]));
    assert_eq!(ranges(0, 0, u64::MAX, 0), vec([brange(128..=191)]));
    assert_eq!(ranges(0, 0, 0, u64::MAX), vec([brange(192..=255)]));
    assert_eq!(
        ranges(u64::MAX, 0, 0, u64::MAX),
        vec([brange(0..=63), brange(192..=255)])
    );
    assert_eq!(
        ranges(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
        vec([
            brange(0..=63),
            brange(64..=127),
            brange(128..=191),
            brange(192..=255)
        ])
    );
    assert_eq!(ranges(1, 0, 0, 0), vec([brange(0)]));
    assert_eq!(
        ranges(0x8000000000000001, 0, 0, 0),
        vec([brange(0), brange(63)])
    );
    assert_eq!(
        ranges(0x8000000000000001, 0x8000000000000001, 0, 0),
        vec([brange(0), brange(63), brange(64), brange(127)])
    );
    assert_eq!(
        ranges(0xC000000000000007, 0x1F000001, 0, 0),
        vec([brange(0..=2), brange(62..=63), brange(64), brange(88..=92)])
    );

    let arena = Arena::new();
    let tr = Transition::epsilon_in(&arena);
    assert_eq!(tr.ranges().next(), None);
}

#[test]
fn transition_contains_symbol() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(b"\x00bcde\xFF", &arena);
    assert_eq!(tr.contains(0), true);
    assert_eq!(tr.contains(255), true);
    assert_eq!(tr.contains(b'b'), true);
    assert_eq!(tr.contains(&b'c'), true);
    assert_eq!(tr.contains(b'f'), false);
    assert_eq!(tr.contains(254), false);
}

#[test]
fn transition_contains_range() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(&[0, 1, 5, 6, 7, 255], &arena);
    assert!(tr.contains(brange(0)));
    assert!(tr.contains(brange(0..=1)));
    assert!(tr.contains(&brange(5..=7)));
    assert!(tr.contains(brange(255)));
    assert!(!tr.contains(brange(0..=3)));
    assert!(!tr.contains(&brange(2..=4)));
    assert!(!tr.contains(brange(254)));
}

#[test]
fn transition_contains_range_inclusive() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(&[0, 1, 5, 6, 7, 255], &arena);
    assert!(tr.contains(0..=0));
    assert!(tr.contains(0..=1));
    assert!(tr.contains(&(5..=7)));
    assert!(tr.contains(255..=255));
    assert!(!tr.contains(0..=3));
    assert!(!tr.contains(&(2..=4)));
    assert!(!tr.contains(254..=254));
}

#[test]
fn transition_contains_transition() {
    let arena = Arena::new();
    let tr_a = Transition::from_symbols_in(b"ace", &arena);
    let tr_b = Transition::from_symbols_in(b"bdf", &arena);
    let tr_c = Transition::from_symbols_in(b"abcdefg", &arena);
    assert!(tr_a.contains(&tr_a));
    assert!(tr_b.contains(&tr_b));
    assert!(tr_c.contains(&tr_c));
    assert!(tr_c.contains(&tr_a));
    assert!(tr_c.contains(&tr_a));
    assert!(!tr_a.contains(&tr_b));
    assert!(!tr_a.contains(&tr_c));
    assert!(!tr_b.contains(&tr_a));
    assert!(!tr_b.contains(&tr_c));
}

#[test]
fn transition_contains_epsilon() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(b"ace", &arena);
    assert!(!tr.contains(Epsilon));
    tr.merge(Epsilon);
    assert!(tr.contains(Epsilon));
}

#[test]
fn transition_intersects_symbol() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(b"\x00bcde\xFF", &arena);
    assert_eq!(tr.intersects(0), true);
    assert_eq!(tr.intersects(255), true);
    assert_eq!(tr.intersects(b'b'), true);
    assert_eq!(tr.intersects(b'c'), true);
    assert_eq!(tr.intersects(&b'f'), false);
    assert_eq!(tr.intersects(&254), false);
}

#[test]
fn transition_intersects_range() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(b"\x00bcde\xFF", &arena);
    assert_eq!(tr.intersects(brange(0..=255)), true);
    assert_eq!(tr.intersects(brange(0)), true);
    assert_eq!(tr.intersects(brange(b'a'..=b'b')), true);
    assert_eq!(tr.intersects(&brange(255)), true);
    assert_eq!(tr.intersects(&brange(102..=254)), false);
    assert_eq!(tr.intersects(254), false);
}

#[test]
fn transition_intersects_range_inclusive() {
    let arena = Arena::new();
    let tr = Transition::from_symbols_in(b"\x00bcde\xFF", &arena);
    assert_eq!(tr.intersects(0..=255), true);
    assert_eq!(tr.intersects(0..=0), true);
    assert_eq!(tr.intersects(b'a'..=b'b'), true);
    assert_eq!(tr.intersects(255..=255), true);
    assert_eq!(tr.intersects(&(102..=254)), false);
    assert_eq!(tr.intersects(&(254..=254)), false);
}

#[test]
fn transition_intersects_transition() {
    let arena = Arena::new();
    let tr_a = Transition::from_symbols_in(b"ace", &arena);
    let tr_b = Transition::from_symbols_in(b"bdf", &arena);
    let tr_c = Transition::from_symbols_in(b"abcde", &arena);
    assert_eq!(tr_a.intersects(&tr_b), false);
    assert_eq!(tr_a.intersects(&tr_c), true);
    assert_eq!(tr_b.intersects(&tr_c), true);
}

#[test]
fn transition_merge_transition() {
    let arena = Arena::new();
    let tr_a = Transition::from_symbols_in(b"abc", &arena);
    let tr_b = Transition::from_symbols_in(b"bcde", &arena);
    let tr_c = Transition::from_symbols_in(b"abcde", &arena);
    tr_a.merge(&tr_b);
    assert_eq!(tr_a, tr_c);
}

#[test]
fn transition_merge_symbol() {
    let arena = Arena::new();
    let a = Transition::new_in(&arena);
    a.merge(64);
    a.merge(&63);
    a.merge(0);
    let mut iter = a.symbols();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(63));
    assert_eq!(iter.next(), Some(64));
    assert_eq!(iter.next(), None);
}

#[test]
fn transition_merge_range() {
    fn check(range: impl Into<RangeU8>) -> Option<RangeU8> {
        let range = range.into();
        let arena = Arena::new();
        let a = Transition::new_in(&arena);
        a.merge(&range);
        a.merge(range);
        let mut range: Option<RangeU8> = None;
        for next_range in a.ranges() {
            range = if let Some(range) = range {
                Some(range.merge(&next_range))
            } else {
                Some(next_range)
            }
        }
        range
    }
    assert_eq!(check(0..=2), Some(brange(0..=2)));
    assert_eq!(check(3..=12), Some(brange(3..=12)));
    assert_eq!(check(0..=63), Some(brange(0..=63)));
    assert_eq!(check(0..=100), Some(brange(0..=100)));
    assert_eq!(check(63..=127), Some(brange(63..=127)));
    assert_eq!(check(63..=200), Some(brange(63..=200)));
    assert_eq!(check(0..=255), Some(brange(0..=255)));
    assert_eq!(check(192..=255), Some(brange(192..=255)));
}

#[test]
fn transition_merge_range_inclusive() {
    fn check(range: impl Into<std::ops::RangeInclusive<u8>>) -> Option<RangeU8> {
        let range = range.into();
        let arena = Arena::new();
        let a = Transition::new_in(&arena);
        a.merge(&range);
        a.merge(range);
        let mut range: Option<RangeU8> = None;
        for next_range in a.ranges() {
            range = if let Some(range) = range {
                Some(range.merge(&next_range))
            } else {
                Some(next_range)
            }
        }
        range
    }
    assert_eq!(check(0..=2), Some(brange(0..=2)));
    assert_eq!(check(3..=12), Some(brange(3..=12)));
    assert_eq!(check(0..=63), Some(brange(0..=63)));
    assert_eq!(check(0..=100), Some(brange(0..=100)));
    assert_eq!(check(63..=127), Some(brange(63..=127)));
    assert_eq!(check(63..=200), Some(brange(63..=200)));
    assert_eq!(check(0..=255), Some(brange(0..=255)));
    assert_eq!(check(192..=255), Some(brange(192..=255)));
}

#[test]
fn transition_display_fmt() {
    fn tr(bytes: &[u8]) -> String {
        let arena = Arena::new();
        format!("{}", Transition::from_symbols_in(bytes, &arena))
    }
    assert_eq!(tr(b""), "[]");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abcE"), "['E' | 'a'-'c']");
    assert_eq!(tr(b"?@"), "['?'-'@']");

    let arena = Arena::new();
    let tr = Transition::new_in(&arena);
    tr.merge(2..=4);
    tr.merge(5..=6);
    assert_eq!(format!("{tr}"), "[02h-06h]");
}

#[test]
fn transition_display_fmt_with_epsilon() {
    let arena = Arena::new();
    assert_eq!(format!("{}", Transition::epsilon_in(&arena)), "[Epsilon]");
    let tr = Transition::from_symbols_in(b"abc", &arena);
    tr.merge(Epsilon);
    assert_eq!(format!("{tr}"), "['a'-'c' | Epsilon]");
    tr.merge(u8::MAX);
    assert_eq!(format!("{tr}"), "['a'-'c' | FFh | Epsilon]");
}

#[test]
fn transition_debug_fmt() {
    fn tr(bytes: &[u8]) -> String {
        let arena = Arena::new();
        format!("{:?}", Transition::from_symbols_in(bytes, &arena))
    }
    assert_eq!(tr(b""), "[]");
    assert_eq!(tr(b"abc"), "[97-99]");
    assert_eq!(tr(b"abc"), "[97-99]");
    assert_eq!(tr(b"abcE"), "[69 | 97-99]");
    assert_eq!(tr(b"?@"), "[63 | 64]");

    let arena = Arena::new();
    let tr = Transition::new_in(&arena);
    tr.merge(2..=4);
    tr.merge(5..=6);
    assert_eq!(format!("{tr}"), "[02h-06h]");
}

#[test]
fn transition_debug_fmt_with_epsilon() {
    let arena = Arena::new();
    assert_eq!(format!("{:?}", Transition::epsilon_in(&arena)), "[Epsilon]");
    let tr = Transition::from_symbols_in(b"?@ABC", &arena);
    tr.merge(Epsilon);
    assert_eq!(format!("{tr:?}"), "[63 | 64-67 | Epsilon]");
    tr.merge(u8::MAX);
    assert_eq!(format!("{tr:?}"), "[63 | 64-67 | 255 | Epsilon]");
}
