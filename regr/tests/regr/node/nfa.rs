use pretty_assertions::{assert_eq, assert_ne};
use regr::adt::Set;
use regr::{edge, Arena};

#[test]
fn node_partial_eq() {
    let arena_1 = Arena::<u8>::new();
    let node_1 = arena_1.node_nfa();
    assert_ne!(node_1, arena_1.node_nfa());

    let arena_2 = Arena::<u8>::new();
    let node_2 = arena_2.node_nfa();
    assert_eq!(node_1, node_2);
}

#[test]
fn node_connect() {
    let arena = Arena::new();
    let node_a = arena.node_nfa();
    let node_b = arena.node_nfa();
    let node_c = arena.node_nfa();
    node_a.connect(node_b, edge!['a']);
    node_a.connect(node_c, edge!['a']);
    node_a.connect(node_c, edge!['a']);
    node_c.connect_with_epsilon(node_a);
}

#[test]
fn node_eclose() {
    let arena = Arena::new();
    let a = arena.node_nfa();
    let b = arena.node_nfa();
    let c = arena.node_nfa();
    let d = arena.node_nfa();

    a.connect(b, edge!['a', char::MAX]);
    a.connect_with_epsilon(b);
    b.connect_with_epsilon(c);
    c.connect(d, edge!['c']);
    b.connect_with_epsilon(a);
    d.connect_with_epsilon(a);
    d.connect_with_epsilon(b);
    d.connect_with_epsilon(c);

    let set = Set::from_iter(vec![a, b, c]);
    assert_eq!(a.eclosure(), set)
}

#[test]
#[cfg_attr(any(feature = "hash-map", feature = "ordered-hash-map"), ignore)]
fn node_print_graph_0() {
    let arena = Arena::new();
    let a = arena.node_nfa();
    let b = arena.node_nfa();
    let c = arena.node_nfa();
    let d = arena.node_nfa();

    a.connect(b, edge!['a'..=char::MAX]);
    a.connect_with_epsilon(b);
    b.connect_with_epsilon(c);
    c.connect(d, edge!['c']);
    b.connect_with_epsilon(a);
    d.connect_with_epsilon(a);
    d.connect_with_epsilon(b);
    d.connect_with_epsilon(c);
    let mut string = String::new();
    a.print_graph(&mut string, "        ").unwrap();
    assert_eq!(
        string,
        "        \
        start 0:
            ['a'-'\\u{10ffff}'] -> 1
            EPSILON -> 1
        node 1:
            EPSILON -> 0
            EPSILON -> 2
        node 2:
            ['c'] -> 3
        node 3:
            EPSILON -> 0
            EPSILON -> 1
            EPSILON -> 2"
    );
}
