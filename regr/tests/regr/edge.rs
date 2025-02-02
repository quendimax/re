use pretty_assertions::assert_eq;
use regr::range;
use regr::{edge, Edge};

#[test]
fn edge_push() {
    let mut edge = Edge::new();
    edge.push('c'..='d');
    edge.push('e'..='e');
    edge.push('y'..='z');
}

#[test]
#[should_panic]
fn edge_push_panic_0() {
    let mut edge = Edge::new();
    edge.push('c'..='d');
    edge.push('d'..='z');
}

#[test]
#[should_panic]
fn edge_push_panic_1() {
    let mut edge = Edge::new();
    edge.push('c'..='d');
    edge.push('a'..='b');
}

#[test]
fn edge_macro() {
    let _: Edge<u32> = edge![];
    let _ = edge!['a'..='b', 'c'..='c', 'e'];
}

#[test]
#[should_panic]
fn edge_macro_panic() {
    let _ = edge!['a'..='b', 'b'..='c'];
}

#[test]
fn edge_partial_eq() {
    assert_eq!(edge!['a', 'c'..='e'], edge!['a', 'c'..='e']);
    assert_eq!(edge!['a', 'c'..=char::MAX], edge!['a', 'c'..=char::MAX]);
    assert_eq!(edge!['a', 'b'..='e'], edge!['a'..='e']);

    // reversed
    assert_eq!(edge!['a', 'c'..='e'], edge!['a', 'c'..='e']);
    assert_eq!(edge!['a', 'c'..=char::MAX], edge!['a', 'c'..=char::MAX]);
    assert_eq!(edge!['a'..='e'], edge!['a', 'b'..='e']);

    assert_ne!(edge!['a', 'c'..='e'], edge!['a', 'b'..='e']);
    assert_ne!(edge!['a', 'c'..='e'], edge!['a', 'd'..='e']);
    assert_ne!(edge!['a', 'c'..='e'], edge!['a', 'c'..='d']);
    assert_ne!(edge!['a', 'c'..='e'], edge!['a', 'c'..='f']);

    assert_ne!(edge!['a', 'b'..='e'], edge!['a', 'c'..='e']);
    assert_ne!(edge!['a', 'd'..='e'], edge!['a', 'c'..='e']);
    assert_ne!(edge!['a', 'c'..='d'], edge!['a', 'c'..='e']);
    assert_ne!(edge!['a', 'c'..='f'], edge!['a', 'c'..='e']);
}

#[test]
fn edge_debug_fmt() {
    assert_eq!(
        format!("{:?}", edge!['a'..='a', 'а'..='ў']),
        r"['a'|'а'-'ў']"
    );
    assert_eq!(
        format!("{:?}", edge!['\0'..='a', 'а'..='ў']),
        r"['\0'-'a'|'а'-'ў']"
    );
}

#[test]
fn edge_merge_range_0() {
    let mut edge = edge!['c'..='e'];
    edge.merge_range(&range('a'));
    assert_eq!(edge, edge!['a', 'c'..='e']);

    let mut edge = edge!['c'..='e'];
    edge.merge_range(&range('g'));
    assert_eq!(edge, edge!['c'..='e', 'g']);

    let mut edge = edge!['c'..='e'];
    edge.merge_range(&range('b'));
    assert_eq!(edge, edge!['b'..='e']);

    let mut edge = edge!['c'..='e'];
    edge.merge_range(&range('f'));
    assert_eq!(edge, edge!['c'..='f']);

    let mut edge = edge!['c'..='e'];
    edge.merge_range(&range('e'));
    assert_eq!(edge, edge!['c'..='e']);

    let mut edge = edge!['c'];
    edge.merge_range(&range('b'));
    assert_eq!(edge, edge!['b'..='c']);

    let mut edge = edge!['c'];
    edge.merge_range(&range('c'));
    assert_eq!(edge, edge!['c']);

    let mut edge = edge!['c'];
    edge.merge_range(&range('d'));
    assert_eq!(edge, edge!['c'..='d']);

    let mut edge = edge!['c'];
    edge.merge_range(&range('a'..='d'));
    assert_eq!(edge, edge!['a'..='d']);
}

#[test]
fn edge_merge_range_1() {
    let mut edge = edge!['c'..='e', 'm'..='n'];
    edge.merge_range(&range('h'));
    assert_eq!(edge, edge!['c'..='e', 'h', 'm'..='n']);

    let mut edge = edge!['c'..='e', 'm'..='n'];
    edge.merge_range(&range('f'..='l'));
    assert_eq!(edge, edge!['c'..='l', 'm'..='n']);

    let mut edge = edge!['c'..='e', 'm'..='n'];
    edge.merge_range(&range('k'..='l'));
    assert_eq!(edge, edge!['c'..='e', 'k'..='n']);

    let mut edge = edge!['c'..='e', 'm'..='n'];
    edge.merge_range(&range('a'..='l'));
    assert_eq!(edge, edge!['a'..='l', 'm'..='n']);

    let mut edge = edge!['c'..='e', 'm'..='n'];
    edge.merge_range(&range('a'..='m'));
    assert_eq!(edge, edge!['a'..='h', 'i'..='n']);

    let mut edge = edge!['c'..='e', 'm'..='n', 'q'];
    edge.merge_range(&range('a'..='v'));
    assert_eq!(edge, edge!['a'..='l', 'm'..='r', 's'..='v']);

    let mut edge = edge!['c'..='e', 'm'..='n', 'q'];
    edge.merge_range(&range('e'..='q'));
    assert_eq!(edge, edge!['c'..='j', 'k'..='n', 'o'..='q']);
}

#[test]
fn edge_merge() {
    let mut edge = edge!['d'..='e', 'm'..='n', 'q', 'ў'];
    edge.merge(&edge!['a', 'c'..='o', 'p'..=char::MAX]);
    assert_eq!(
        edge,
        edge![
            'a',
            'c'..='i',
            'j'..='o',
            'p'..='\u{88038}',
            '\u{88039}'..=char::MAX
        ]
    );
}

#[test]
fn edge_fold() {
    let mut edge = edge!['a'..='b', 'c'..='d', 'q', 'r', 'ў'];
    let original = edge.clone();
    edge.fold();
    assert_eq!(edge, edge!['a'..='d', 'q'..='r', 'ў']);
    assert_eq!(edge, original);
}
