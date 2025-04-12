use pretty_assertions::assert_eq;
use regr::range;
use regr::{Edge, edge};

#[test]
fn edge_push() {
    let mut edge = Edge::new();
    edge.push(b'c'..=b'd');
    edge.push(b'e'..=b'e');
    edge.push(b'y'..=b'z');
}

#[test]
#[should_panic]
fn edge_push_panic_0() {
    let mut edge = Edge::new();
    edge.push(b'c'..=b'd');
    edge.push(b'd'..=b'z');
}

#[test]
#[should_panic]
fn edge_push_panic_1() {
    let mut edge = Edge::new();
    edge.push(b'c'..=b'd');
    edge.push(b'a'..=b'b');
}

#[test]
fn edge_macro() {
    let _ = edge![];
    let _ = edge![b'a'..=b'b', b'c'..=b'c', b'e'];
}

#[test]
#[should_panic]
fn edge_macro_panic() {
    let _ = edge![b'a'..=b'b', b'b'..=b'c'];
}

#[test]
fn edge_partial_eq() {
    assert_eq!(edge![b'a', b'c'..=b'e'], edge![b'a', b'c'..=b'e']);
    assert_eq!(edge![b'a', b'c'..=u8::MAX], edge![b'a', b'c'..=u8::MAX]);
    assert_eq!(edge![b'a', b'b'..=b'e'], edge![b'a'..=b'e']);

    // reversed
    assert_eq!(edge![b'a', b'c'..=b'e'], edge![b'a', b'c'..=b'e']);
    assert_eq!(edge![b'a', b'c'..=u8::MAX], edge![b'a', b'c'..=u8::MAX]);
    assert_eq!(edge![b'a'..=b'e'], edge![b'a', b'b'..=b'e']);

    assert_ne!(edge![b'a', b'c'..=b'e'], edge![b'a', b'b'..=b'e']);
    assert_ne!(edge![b'a', b'c'..=b'e'], edge![b'a', b'd'..=b'e']);
    assert_ne!(edge![b'a', b'c'..=b'e'], edge![b'a', b'c'..=b'd']);
    assert_ne!(edge![b'a', b'c'..=b'e'], edge![b'a', b'c'..=b'f']);

    assert_ne!(edge![b'a', b'b'..=b'e'], edge![b'a', b'c'..=b'e']);
    assert_ne!(edge![b'a', b'd'..=b'e'], edge![b'a', b'c'..=b'e']);
    assert_ne!(edge![b'a', b'c'..=b'd'], edge![b'a', b'c'..=b'e']);
    assert_ne!(edge![b'a', b'c'..=b'f'], edge![b'a', b'c'..=b'e']);
}

#[test]
fn edge_debug_fmt() {
    assert_eq!(
        format!("{:?}", edge![b'a'..=b'a', b'{'..=b'~']),
        r"['a'|'{'-'~']"
    );
    assert_eq!(
        format!("{:?}", edge![b'\0'..=b'a', b'{'..=b'~']),
        r"[0-'a'|'{'-'~']"
    );
}

#[test]
fn edge_merge_range_0() {
    let mut edge = edge![b'c'..=b'e'];
    edge.merge_range(range(b'a'));
    assert_eq!(edge, edge![b'a', b'c'..=b'e']);

    let mut edge = edge![b'c'..=b'e'];
    edge.merge_range(range(b'g'));
    assert_eq!(edge, edge![b'c'..=b'e', b'g']);

    let mut edge = edge![b'c'..=b'e'];
    edge.merge_range(range(b'b'));
    assert_eq!(edge, edge![b'b'..=b'e']);

    let mut edge = edge![b'c'..=b'e'];
    edge.merge_range(range(b'f'));
    assert_eq!(edge, edge![b'c'..=b'f']);

    let mut edge = edge![b'c'..=b'e'];
    edge.merge_range(range(b'e'));
    assert_eq!(edge, edge![b'c'..=b'e']);

    let mut edge = edge![b'c'];
    edge.merge_range(range(b'b'));
    assert_eq!(edge, edge![b'b'..=b'c']);

    let mut edge = edge![b'c'];
    edge.merge_range(range(b'c'));
    assert_eq!(edge, edge![b'c']);

    let mut edge = edge![b'c'];
    edge.merge_range(range(b'd'));
    assert_eq!(edge, edge![b'c'..=b'd']);

    let mut edge = edge![b'c'];
    edge.merge_range(range(b'a'..=b'd'));
    assert_eq!(edge, edge![b'a'..=b'd']);
}

#[test]
fn edge_merge_range_1() {
    let mut edge = edge![b'c'..=b'e', b'm'..=b'n'];
    edge.merge_range(range(b'h'));
    assert_eq!(edge, edge![b'c'..=b'e', b'h', b'm'..=b'n']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n'];
    edge.merge_range(range(b'f'..=b'l'));
    assert_eq!(edge, edge![b'c'..=b'l', b'm'..=b'n']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n'];
    edge.merge_range(range(b'k'..=b'l'));
    assert_eq!(edge, edge![b'c'..=b'e', b'k'..=b'n']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n'];
    edge.merge_range(range(b'a'..=b'l'));
    assert_eq!(edge, edge![b'a'..=b'l', b'm'..=b'n']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n'];
    edge.merge_range(range(b'a'..=b'm'));
    assert_eq!(edge, edge![b'a'..=b'h', b'i'..=b'n']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n', b'q'];
    edge.merge_range(range(b'a'..=b'v'));
    assert_eq!(edge, edge![b'a'..=b'l', b'm'..=b'r', b's'..=b'v']);

    let mut edge = edge![b'c'..=b'e', b'm'..=b'n', b'q'];
    edge.merge_range(range(b'e'..=b'q'));
    assert_eq!(edge, edge![b'c'..=b'j', b'k'..=b'n', b'o'..=b'q']);
}

#[test]
fn edge_merge() {
    let mut edge = edge![b'd'..=b'e', b'm'..=b'n', b'q', b'{'];
    edge.merge(&edge![b'a', b'c'..=b'o', b'p'..=u8::MAX]);
    assert_eq!(
        edge,
        edge![b'a', b'c'..=b'i', b'j'..=b'o', b'p'..=b'z', b'{'..=u8::MAX]
    );
}

#[test]
fn edge_fold() {
    let mut edge = edge![b'a'..=b'b', b'c'..=b'd', b'q', b'r', b'~'];
    let original = edge.clone();
    edge.fold();
    assert_eq!(edge, edge![b'a'..=b'd', b'q'..=b'r', b'~']);
    assert_eq!(edge, original);
}
