use pretty_assertions::assert_eq;
use regr::{Range, Transition, range};

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
