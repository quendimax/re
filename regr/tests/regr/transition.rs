use pretty_assertions::assert_eq;
use regr::{Range, Transition, range};

#[test]
fn transition_new() {
    let transition = Transition::new(&[0, 1, 2, 3]);
    assert_eq!(
        transition.symbols().collect::<Vec<_>>(),
        vec![64, 129, 192, 193]
    );
}

#[test]
fn transition_from_bytes() {
    let transition = Transition::from_bytes(b"\0abc\xFF");
    assert_eq!(
        transition.symbols().collect::<Vec<_>>(),
        vec![0, 97, 98, 99, 255]
    );
}

#[test]
fn transition_symbols() {
    type Vec = smallvec::SmallVec<[u8; 8]>;
    fn symbols(a: u64, b: u64, c: u64, d: u64) -> Vec {
        let smap = Transition::new(&[a, b, c, d]);
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
}

#[test]
fn transition_ranges() {
    type Vec = smallvec::SmallVec<[Range; 4]>;
    fn ranges(a: u64, b: u64, c: u64, d: u64) -> Vec {
        let smap = Transition::new(&[a, b, c, d]);
        smap.ranges().collect::<Vec>()
    }
    fn vec<const N: usize>(buf: [Range; N]) -> Vec {
        Vec::from(&buf as &[Range])
    }

    assert_eq!(ranges(0, 0, 0, 0), vec([]));
    assert_eq!(ranges(255, 0, 0, 0), vec([range(0..=7)]));
    assert_eq!(ranges(255, 255, 0, 0), vec([range(0..=7), range(64..=71)]));
    assert_eq!(ranges(0, 255, 0, 0), vec([range(64..=71)]));
    assert_eq!(ranges(0, 0, 0, 255), vec([range(192..=199)]));
    assert_eq!(
        ranges(255, 255, 255, 255),
        vec([
            range(0..=7),
            range(64..=71),
            range(128..=135),
            range(192..=199)
        ])
    );
    assert_eq!(ranges(u64::MAX, 0, 0, 0), vec([range(0..=63)]));
    assert_eq!(ranges(0, u64::MAX, 0, 0), vec([range(64..=127)]));
    assert_eq!(ranges(0, 0, u64::MAX, 0), vec([range(128..=191)]));
    assert_eq!(ranges(0, 0, 0, u64::MAX), vec([range(192..=255)]));
    assert_eq!(
        ranges(u64::MAX, 0, 0, u64::MAX),
        vec([range(0..=63), range(192..=255)])
    );
    assert_eq!(
        ranges(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
        vec([
            range(0..=63),
            range(64..=127),
            range(128..=191),
            range(192..=255)
        ])
    );
    assert_eq!(ranges(1, 0, 0, 0), vec([range(0)]));
    assert_eq!(
        ranges(0x8000000000000001, 0, 0, 0),
        vec([range(0), range(63)])
    );
    assert_eq!(
        ranges(0x8000000000000001, 0x8000000000000001, 0, 0),
        vec([range(0), range(63), range(64), range(127)])
    );
    assert_eq!(
        ranges(0xC000000000000007, 0x1F000001, 0, 0),
        vec([range(0..=2), range(62..=63), range(64), range(88..=92)])
    );
}

#[test]
fn transition_merge_transition() {
    let mut tr_a = Transition::from_bytes(b"abc");
    let tr_b = Transition::from_bytes(b"bcde");
    let tr_c = Transition::from_bytes(b"abcde");
    tr_a.merge(&tr_b);
    assert_eq!(tr_a, tr_c);
}

#[test]
fn transition_merge_symbol() {
    let mut a = Transition::default();
    a.merge(64);
    a.merge(63);
    a.merge(0);
    let mut iter = a.symbols();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(63));
    assert_eq!(iter.next(), Some(64));
    assert_eq!(iter.next(), None);
}

#[test]
fn transition_merge_range() {
    fn check(range: impl Into<Range>) -> Option<Range> {
        let range = range.into();
        let mut a = Transition::default();
        a.merge(range);
        let mut range: Option<Range> = None;
        for next_range in a.ranges() {
            range = if let Some(range) = range {
                Some(range.merge(next_range))
            } else {
                Some(next_range)
            }
        }
        range
    }
    assert_eq!(check(0..=2), Some(range(0..=2)));
    assert_eq!(check(3..=12), Some(range(3..=12)));
    assert_eq!(check(0..=63), Some(range(0..=63)));
    assert_eq!(check(0..=100), Some(range(0..=100)));
    assert_eq!(check(63..=127), Some(range(63..=127)));
    assert_eq!(check(63..=200), Some(range(63..=200)));
    assert_eq!(check(0..=255), Some(range(0..=255)));
    assert_eq!(check(192..=255), Some(range(192..=255)));
}

#[test]
fn transition_display_fmt() {
    fn tr(bytes: &[u8]) -> String {
        format!("{}", Transition::from_bytes(bytes))
    }
    assert_eq!(tr(b""), "[]");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abcE"), "['E' | 'a'-'c']");
}
