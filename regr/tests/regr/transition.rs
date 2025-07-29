use pretty_assertions::assert_eq;
use redt::{RangeU8, range};
use regr::{Arena, Epsilon, Graph, Transition};

fn single(sym: u8) -> RangeU8 {
    range(sym, sym)
}

fn handle_tr<F, R>(f: F) -> R
where
    F: Fn(Transition<'_>) -> R,
{
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    let tr = gr.node().connect(gr.node());
    f(tr)
}

fn handle_tr_from_chunks<F, R>(chunks: &[u64; 4], f: F) -> R
where
    F: Fn(Transition<'_>) -> R,
{
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    let tr = gr.node().connect(gr.node());
    let mut sym = 0u8;
    for chunk in chunks {
        let mut mask = 1u64;
        for _ in 0..64 {
            if mask & *chunk != 0 {
                tr.merge(sym);
            }
            mask <<= 1;
            sym = sym.wrapping_add(1);
        }
    }
    f(tr)
}

fn handle_tr_from_symbols<F, R>(symbols: &[u8], f: F) -> R
where
    F: Fn(Transition<'_>) -> R,
{
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    let tr = gr.node().connect(gr.node());
    for sym in symbols {
        tr.merge(*sym);
    }
    f(tr)
}

fn handle_epsilon<F, R>(f: F) -> R
where
    F: Fn(Transition<'_>) -> R,
{
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    let tr = gr.node().connect(gr.node());
    tr.merge(Epsilon);
    f(tr)
}

#[test]
fn transition_new() {
    handle_tr(|tr| assert_eq!(tr.symbols().collect::<Vec<_>>(), vec![]));
}

#[test]
fn transition_symbols() {
    type Vec = smallvec::SmallVec<[u8; 8]>;
    fn symbols(a: u64, b: u64, c: u64, d: u64) -> Vec {
        handle_tr_from_chunks(&[a, b, c, d], |tr| tr.symbols().collect::<Vec>())
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

    handle_epsilon(|tr| assert_eq!(tr.symbols().next(), None));
}

#[test]
fn transition_ranges() {
    type Vec = smallvec::SmallVec<[RangeU8; 4]>;
    fn ranges(a: u64, b: u64, c: u64, d: u64) -> Vec {
        handle_tr_from_chunks(&[a, b, c, d], |tr| tr.ranges().collect::<Vec>())
    }
    fn vec<const N: usize>(buf: [RangeU8; N]) -> Vec {
        Vec::from(&buf as &[RangeU8])
    }

    assert_eq!(ranges(0, 0, 0, 0), vec([]));
    assert_eq!(ranges(255, 0, 0, 0), vec([range(0, 7)]));
    assert_eq!(ranges(255, 255, 0, 0), vec([range(0, 7), range(64, 71)]));
    assert_eq!(ranges(0, 255, 0, 0), vec([range(64, 71)]));
    assert_eq!(ranges(0, 0, 0, 255), vec([range(192, 199)]));
    assert_eq!(
        ranges(255, 255, 255, 255),
        vec([range(0, 7), range(64, 71), range(128, 135), range(192, 199)])
    );
    assert_eq!(ranges(u64::MAX, 0, 0, 0), vec([range(0, 63)]));
    assert_eq!(ranges(0, u64::MAX, 0, 0), vec([range(64, 127)]));
    assert_eq!(ranges(0, 0, u64::MAX, 0), vec([range(128, 191)]));
    assert_eq!(ranges(0, 0, 0, u64::MAX), vec([range(192, 255)]));
    assert_eq!(
        ranges(u64::MAX, 0, 0, u64::MAX),
        vec([range(0, 63), range(192, 255)])
    );
    assert_eq!(
        ranges(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
        vec([
            range(0, 63),
            range(64, 127),
            range(128, 191),
            range(192, 255)
        ])
    );
    assert_eq!(ranges(1, 0, 0, 0), vec([single(0)]));
    assert_eq!(
        ranges(0x8000000000000001, 0, 0, 0),
        vec([single(0), single(63)])
    );
    assert_eq!(
        ranges(0x8000000000000001, 0x8000000000000001, 0, 0),
        vec([single(0), single(63), single(64), single(127)])
    );
    assert_eq!(
        ranges(0xC000000000000007, 0x1F000001, 0, 0),
        vec([range(0, 2), range(62, 63), single(64), range(88, 92)])
    );

    handle_epsilon(|tr| assert_eq!(tr.ranges().next(), None));
}

#[test]
fn transition_contains_symbol() {
    handle_tr_from_symbols(b"\x00bcde\xFF", |tr| {
        assert_eq!(tr.contains(0), true);
        assert_eq!(tr.contains(255), true);
        assert_eq!(tr.contains(b'b'), true);
        assert_eq!(tr.contains(&b'c'), true);
        assert_eq!(tr.contains(b'f'), false);
        assert_eq!(tr.contains(254), false);
    });
}

#[test]
fn transition_contains_range() {
    handle_tr_from_symbols(&[0, 1, 5, 6, 7, 255], |tr| {
        assert!(tr.contains(single(0)));
        assert!(tr.contains(range(0, 1)));
        assert!(tr.contains(&range(5, 7)));
        assert!(tr.contains(single(255)));
        assert!(!tr.contains(range(0, 3)));
        assert!(!tr.contains(&range(2, 4)));
        assert!(!tr.contains(single(254)));
    });
}

#[test]
fn transition_contains_transition() {
    let mut arena = Arena::new();
    let gr = Graph::dfa_in(&mut arena);
    let tr_a = gr.node().connect(gr.node());
    tr_a.merge(b'a');
    tr_a.merge(b'c');
    tr_a.merge(b'e');
    let tr_b = gr.node().connect(gr.node());
    tr_b.merge(b'b');
    tr_b.merge(b'd');
    tr_b.merge(b'f');
    let tr_c = gr.node().connect(gr.node());
    tr_c.merge(b'a');
    tr_c.merge(b'b');
    tr_c.merge(b'c');
    tr_c.merge(b'd');
    tr_c.merge(b'e');
    tr_c.merge(b'f');
    tr_c.merge(b'g');
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
    handle_tr_from_symbols(b"ace", |tr| {
        assert!(!tr.contains(Epsilon));
        tr.merge(Epsilon);
        assert!(tr.contains(Epsilon));
    });
}

#[test]
fn transition_intersects_symbol() {
    handle_tr_from_symbols(b"\x00bcde\xFF", |tr| {
        assert_eq!(tr.intersects(0), true);
        assert_eq!(tr.intersects(255), true);
        assert_eq!(tr.intersects(b'b'), true);
        assert_eq!(tr.intersects(b'c'), true);
        assert_eq!(tr.intersects(&b'f'), false);
        assert_eq!(tr.intersects(&254), false);
    });
}

#[test]
fn transition_intersects_range() {
    handle_tr_from_symbols(b"\x00bcde\xFF", |tr| {
        assert_eq!(tr.intersects(range(0, 255)), true);
        assert_eq!(tr.intersects(single(0)), true);
        assert_eq!(tr.intersects(range(b'a', b'b')), true);
        assert_eq!(tr.intersects(&single(255)), true);
        assert_eq!(tr.intersects(&range(102, 254)), false);
        assert_eq!(tr.intersects(254), false);
    });
}

#[test]
fn transition_intersects_transition() {
    let mut arena = Arena::new();
    let gr = Graph::dfa_in(&mut arena);
    let tr_a = gr.node().connect(gr.node());
    tr_a.merge(b'a');
    tr_a.merge(b'c');
    tr_a.merge(b'e');
    let tr_b = gr.node().connect(gr.node());
    tr_b.merge(b'b');
    tr_b.merge(b'd');
    tr_b.merge(b'f');
    let tr_c = gr.node().connect(gr.node());
    tr_c.merge(b'a');
    tr_c.merge(b'b');
    tr_c.merge(b'c');
    tr_c.merge(b'd');
    tr_c.merge(b'e');
    tr_c.merge(b'f');
    assert_eq!(tr_a.intersects(&tr_b), false);
    assert_eq!(tr_a.intersects(&tr_c), true);
    assert_eq!(tr_b.intersects(&tr_c), true);
}

#[test]
fn transition_merge_transition() {
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    let tr_a = gr.node().connect(gr.node());
    tr_a.merge(b'a');
    tr_a.merge(b'b');
    tr_a.merge(b'c');
    let tr_b = gr.node().connect(gr.node());
    tr_b.merge(b'b');
    tr_b.merge(b'c');
    tr_b.merge(b'd');
    tr_b.merge(b'e');
    let tr_c = gr.node().connect(gr.node());
    tr_c.merge(b'a');
    tr_c.merge(b'b');
    tr_c.merge(b'c');
    tr_c.merge(b'd');
    tr_c.merge(b'e');

    tr_a.merge(tr_b);
    assert_eq!(tr_a, tr_c);
}

#[test]
fn transition_merge_symbol() {
    handle_tr(|tr| {
        tr.merge(64);
        tr.merge(63);
        tr.merge(0);
        let mut iter = tr.symbols();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(63));
        assert_eq!(iter.next(), Some(64));
        assert_eq!(iter.next(), None);
    });
}

#[test]
fn transition_merge_range() {
    fn check(range: impl Into<RangeU8>) -> Option<RangeU8> {
        let range = range.into();
        let mut arena = Arena::new();
        let gr = Graph::nfa_in(&mut arena);
        let tr = gr.node().connect(gr.node());
        tr.merge(range);
        let mut range: Option<RangeU8> = None;
        for next_range in tr.ranges() {
            range = if let Some(range) = range {
                Some(range.merge(&next_range))
            } else {
                Some(next_range)
            }
        }
        range
    }
    assert_eq!(check(0..=2), Some(range(0, 2)));
    assert_eq!(check(3..=12), Some(range(3, 12)));
    assert_eq!(check(0..=63), Some(range(0, 63)));
    assert_eq!(check(0..=100), Some(range(0, 100)));
    assert_eq!(check(63..=127), Some(range(63, 127)));
    assert_eq!(check(63..=200), Some(range(63, 200)));
    assert_eq!(check(0..=255), Some(range(0, 255)));
    assert_eq!(check(192..=255), Some(range(192, 255)));
}

#[test]
fn transition_display_fmt() {
    fn tr(bytes: &[u8]) -> String {
        handle_tr_from_symbols(bytes, |tr| format!("{tr}"))
    }
    assert_eq!(tr(b""), "[]");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abc"), "['a'-'c']");
    assert_eq!(tr(b"abcE"), "['E' | 'a'-'c']");
    assert_eq!(tr(b"?@"), "['?'-'@']");

    handle_tr(|tr| {
        tr.merge(range(2, 4));
        tr.merge(range(5, 6));
        assert_eq!(format!("{tr}"), "[02h-06h]");
    });
}

#[test]
fn transition_display_fmt_with_epsilon() {
    handle_epsilon(|tr| assert_eq!(format!("{}", tr), "[Epsilon]"));
    handle_tr_from_symbols(b"abc", |tr| {
        tr.merge(Epsilon);
        assert_eq!(format!("{tr}"), "['a'-'c' | Epsilon]");
        tr.merge(u8::MAX);
        assert_eq!(format!("{tr}"), "['a'-'c' | FFh | Epsilon]");
    });
}

#[test]
fn transition_debug_fmt() {
    fn tr(bytes: &[u8]) -> String {
        handle_tr_from_symbols(bytes, |tr| format!("{tr:?}"))
    }
    assert_eq!(tr(b""), "[]");
    assert_eq!(tr(b"abc"), "[97-99]");
    assert_eq!(tr(b"abc"), "[97-99]");
    assert_eq!(tr(b"abcE"), "[69 | 97-99]");
    assert_eq!(tr(b"?@"), "[63 | 64]");

    handle_tr(|tr| {
        tr.merge(range(2, 4));
        tr.merge(range(5, 6));
        assert_eq!(format!("{tr}"), "[02h-06h]");
    });
}

#[test]
fn transition_debug_fmt_with_epsilon() {
    handle_epsilon(|tr| assert_eq!(format!("{tr:?}"), "[Epsilon]"));
    handle_tr_from_symbols(b"?@ABC", |tr| {
        tr.merge(Epsilon);
        assert_eq!(format!("{tr:?}"), "[63 | 64-67 | Epsilon]");
        tr.merge(u8::MAX);
        assert_eq!(format!("{tr:?}"), "[63 | 64-67 | 255 | Epsilon]");
    });
}
