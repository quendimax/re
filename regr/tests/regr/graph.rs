use pretty_assertions::assert_eq;
use redt::lit;
use redt::{RangeU8, range};
use regr::{Arena, AutomatonKind, Epsilon, Graph, Tag, TagBank};

#[test]
fn graph_ctor() {
    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);
    assert!(gr.is_nfa());
    assert!(!gr.is_dfa());

    let mut arena = Arena::with_capacity(150);
    let gr = Graph::new_in(&mut arena, AutomatonKind::DFA);
    assert!(!gr.is_nfa());
    assert!(gr.is_dfa());
}

#[test]
fn graph_node() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena, AutomatonKind::NFA);
    assert_eq!(graph.node().nid(), 0);
    assert_eq!(graph.node().nid(), 1);
    assert_eq!(graph.node().nid(), 2);
    drop(graph);

    let mut arena = Arena::with_capacity(9);
    let graph = Graph::new_in(&mut arena, AutomatonKind::NFA);
    assert_eq!(graph.node().nid(), 0);
    assert_eq!(graph.node().nid(), 1);
    assert_eq!(graph.node().nid(), 2);
}

#[test]
fn graph_start_node() {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    assert_eq!(graph.start_node().nid(), 0);
    assert_eq!(graph.node().nid(), 1);
    assert_eq!(graph.node().nid(), 2);
    drop(graph);

    let graph = Graph::nfa_in(&mut arena);
    assert_eq!(graph.node(), graph.start_node());
    assert_eq!(graph.start_node().nid(), 0);
    assert_eq!(graph.node().nid(), 1);
    assert_eq!(graph.node().nid(), 2);
    assert_eq!(graph.start_node().nid(), 0);
}

#[test]
fn graph_arena() {
    let mut arena = Arena::new();
    let arena_ptr = &arena as *const Arena;
    let graph = Graph::nfa_in(&mut arena);
    assert_eq!(graph.arena() as *const Arena, arena_ptr);
}

#[test]
fn graph_determine_0() {
    let mut arena = Arena::new();
    let nfa = Graph::nfa_in(&mut arena);
    let a = nfa.node();
    a.connect(a).merge(Epsilon);
    assert_eq!(
        nfa.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> self
            ///}
        )
    );

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determinize_in(&mut dfa_arena);
    assert_eq!(format!("{dfa}"), "node(0) {}");
}

#[test]
fn graph_determine_1() {
    let mut arena = Arena::new();
    let nfa = Graph::nfa_in(&mut arena);
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    a.connect(a).merge(range(1, 255));
    a.connect(b).merge(Epsilon);
    b.connect(c).merge(b'a');
    c.connect(d).merge(b'b');
    assert_eq!(
        nfa.to_string(),
        lit!(
            ///node(0) {
            ///    [01h-FFh] -> self
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    ['b'] -> node(3)
            ///}
            ///node(3) {}
        )
    );

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determinize_in(&mut dfa_arena);
    assert_eq!(
        dfa.to_string(),
        lit!(
            ///node(0) {
            ///    [01h-'`' | 'b'-FFh] -> self
            ///    ['a'] -> node(1)
            ///}
            ///node(1) {
            ///    [01h-'`' | 'c'-FFh] -> node(0)
            ///    ['a'] -> self
            ///    ['b'] -> node(2)
            ///}
            ///node(2) {
            ///    [01h-'`' | 'b'-FFh] -> node(0)
            ///    ['a'] -> node(1)
            ///}
        )
    );
}

#[test]
fn graph_determine_klenee_star() {
    let mut arena = Arena::new();
    let nfa = Graph::nfa_in(&mut arena);
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    a.connect(b).merge(Epsilon);
    a.connect(d).merge(Epsilon);
    b.connect(c).merge(b'a');
    c.connect(b).merge(Epsilon);
    c.connect(d).merge(Epsilon);
    assert_eq!(
        nfa.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(3)
            ///}
            ///node(1) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(3)
            ///}
            ///node(3) {}
        )
    );

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determinize_in(&mut dfa_arena);
    assert_eq!(
        dfa.to_string(),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(1)
            ///}
            ///node(1) {
            ///    ['a'] -> self
            ///}
        )
    );
}

#[test]
fn graph_for_each_node() {
    let mut arena = Arena::with_capacity(1);
    let graph = Graph::new_in(&mut arena, AutomatonKind::NFA);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b).merge(RangeU8::new(b'a', u8::MAX));
    a.connect(b).merge(Epsilon);
    b.connect(c).merge(Epsilon);
    c.connect(d).merge(b'c');
    b.connect(a).merge(Epsilon);
    d.connect(a).merge(Epsilon);
    d.connect(b).merge(Epsilon);
    d.connect(c).merge(Epsilon);

    #[allow(clippy::mutable_key_type)]
    let mut visited = std::collections::HashSet::new();
    graph.for_each_node(|node| {
        visited.insert(node);
    });
    assert_eq!(visited.len(), 4);
}

#[test]
fn graph_display_fmt_0() {
    let mut arena = Arena::with_capacity(1);
    let graph = Graph::new_in(&mut arena, AutomatonKind::NFA);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b).merge(RangeU8::new(b'a', u8::MAX));
    a.connect(b).merge(Epsilon);
    b.connect(c).merge(Epsilon);
    c.connect(d).merge(b'c');
    b.connect(a).merge(Epsilon);
    d.connect(a).merge(Epsilon);
    d.connect(b).merge(Epsilon);
    d.connect(c).merge(Epsilon);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    ['a'-FFh | Epsilon] -> node(1)
            ///}
            ///node(1) {
            ///    [Epsilon] -> node(0)
            ///    [Epsilon] -> node(2)
            ///}
            ///node(2) {
            ///    ['c'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(0)
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(2)
            ///}
        )
    );
}

#[test]
fn graph_display_fmt_1() {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let n0 = graph.node();
    let n1 = graph.node();
    let n2 = graph.node();
    let n3 = graph.node();
    let n4 = graph.node();
    n0.connect(n1).merge(RangeU8::from(b'a'..=b'b'));
    n0.connect(n1).merge(RangeU8::from(b'd'..=b'z'));
    n1.connect(n2).merge(Epsilon);
    n1.connect(n4).merge(Epsilon);
    n2.connect(n3).merge(b'a');
    n3.connect(n4).merge(Epsilon);
    n3.connect(n2).merge(Epsilon);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    ['a'-'b' | 'd'-'z'] -> node(1)
            ///}
            ///node(1) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(4)
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {}
        )
    );
}

#[test]
fn graph_display_fmt_2() {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let n0 = graph.node();
    let n1 = graph.node();
    let n2 = graph.node();
    let n3 = graph.node();
    let n4 = graph.node();
    let n5 = graph.node();
    let n6 = graph.node();
    let n7 = graph.node();
    n0.connect(n2).merge(Epsilon);
    n0.connect(n5).merge(Epsilon);
    n2.connect(n3).merge(b'a');
    n3.connect(n4).merge(b'b');
    n4.connect(n1).merge(Epsilon);
    n5.connect(n6).merge(b'c');
    n6.connect(n7).merge(b'd');
    n7.connect(n1).merge(Epsilon);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(5)
            ///}
            ///node(1) {}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    ['b'] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(5) {
            ///    ['c'] -> node(6)
            ///}
            ///node(6) {
            ///    ['d'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
fn graph_display_fmt_3() {
    let mut arena = Arena::new();
    let graph = Graph::dfa_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    a.connect(b).merge(1);
    b.connect(b).merge(3);
    b.connect(c).merge(1);
    assert_eq!(
        format!("{graph:?}"),
        lit!(
            ///node(0) {
            ///    [1] -> node(1)
            ///}
            ///node(1) {
            ///    [3] -> self
            ///    [1] -> node(2)
            ///}
            ///node(2) {}
        )
    );
}

#[test]
fn graph_tags() {
    let mut arena = Arena::new();
    let graph = Graph::dfa_in(&mut arena);
    let mut tag_bank = TagBank::default();
    graph.add_tag_group(0, tag_bank.absolute(), tag_bank.absolute());
    graph.add_tag_group(1, tag_bank.absolute(), tag_bank.absolute());

    let mut tag_bank = TagBank::default();
    assert_eq!(
        graph.tag_group(0),
        Some((tag_bank.absolute(), tag_bank.absolute()))
    );
    assert_eq!(graph.tag_group(2), None);
    assert_eq!(
        graph.tag_groups().collect::<Vec<_>>(),
        [
            (
                0,
                (
                    Tag::Absolute { id: 0, reg: 0 },
                    Tag::Absolute { id: 1, reg: 1 },
                ),
            ),
            (
                1,
                (
                    Tag::Absolute { id: 2, reg: 2 },
                    Tag::Absolute { id: 3, reg: 3 },
                ),
            ),
        ]
    );
}
