use pretty_assertions::assert_eq;
use regr::{Arena, AutomatonKind, Epsilon, Graph, Span, span};

fn dsp<T: std::fmt::Display>(obj: &T) -> String {
    let mut result = String::new();
    for line in format!("{obj}").split('\n') {
        if !line.ends_with('{') && !line.ends_with('}') {
            result.push_str("    ");
        }
        result.push_str(line.trim());
        result.push('\n');
    }
    result.trim().to_string()
}

fn dbg<T: std::fmt::Debug>(obj: &T) -> String {
    let mut result = String::new();
    for line in format!("{obj:?}").split('\n') {
        if !line.ends_with('{') && !line.ends_with('}') {
            result.push_str("    ");
        }
        result.push_str(line.trim());
        result.push('\n');
    }
    result.trim().to_string()
}

#[test]
fn graph_ctor() {
    let mut arena = Arena::new();
    _ = Graph::nfa_in(&mut arena);
    let mut arena = Arena::with_capacity(150);
    _ = Graph::new_in(&mut arena, AutomatonKind::NFA);
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
fn graph_determined_0() {
    let mut arena = Arena::new();
    let nfa = Graph::nfa_in(&mut arena);
    let a = nfa.node();
    a.connect(a, Epsilon);
    assert_eq!(
        dsp(&nfa),
        dsp(&"
            node(0) {
                [Epsilon] -> self
            }
        ")
    );

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determine_in(&mut dfa_arena);
    assert_eq!(format!("{dfa}"), "node(0) {}");
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_determined_1() {
    let mut arena = Arena::new();
    let nfa = Graph::nfa_in(&mut arena);
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    a.connect(a, 1..=255);
    a.connect(b, Epsilon);
    b.connect(c, b'a');
    c.connect(d, b'b');
    assert_eq!(
        dsp(&nfa),
        dsp(&"
            node(0) {
                [01h-FFh] -> self
                [Epsilon] -> node(1)
            }
            node(1) {
                ['a'] -> node(2)
            }
            node(2) {
                ['b'] -> node(3)
            }
            node(3) {}
        ")
    );

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determine_in(&mut dfa_arena);
    assert_eq!(
        dsp(&dfa),
        dsp(&"
            node(0) {
                [01h-'`' | 'b'-FFh] -> self
                ['a'] -> node(1)
            }
            node(1) {
                [01h-'`' | 'c'-FFh] -> node(0)
                ['a'] -> self
                ['b'] -> node(2)
            }
            node(2) {
                [01h-'`' | 'b'-FFh] -> node(0)
                ['a'] -> node(1)
            }
        ")
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_0() {
    let mut arena = Arena::with_capacity(1);
    let graph = Graph::new_in(&mut arena, AutomatonKind::NFA);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, Span::new(b'a', u8::MAX));
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    c.connect(d, b'c');
    b.connect(a, Epsilon);
    d.connect(a, Epsilon);
    d.connect(b, Epsilon);
    d.connect(c, Epsilon);
    assert_eq!(
        dsp(&graph),
        dsp(&"
            node(0) {
                ['a'-FFh | Epsilon] -> node(1)
            }
            node(1) {
                [Epsilon] -> node(2)
                [Epsilon] -> node(0)
            }
            node(2) {
                ['c'] -> node(3)
            }
            node(3) {
                [Epsilon] -> node(0)
                [Epsilon] -> node(1)
                [Epsilon] -> node(2)
            }
        ")
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_1() {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let n0 = graph.node();
    let n1 = graph.node();
    let n2 = graph.node();
    let n3 = graph.node();
    let n4 = graph.node();
    n0.connect(n1, span(b'a'..=b'b'));
    n0.connect(n1, span(b'd'..=b'z'));
    n1.connect(n2, Epsilon);
    n1.connect(n4, Epsilon);
    n2.connect(n3, b'a');
    n3.connect(n4, Epsilon);
    n3.connect(n2, Epsilon);
    assert_eq!(
        dsp(&graph),
        dsp(&"
        node(0) {
            ['a'-'b' | 'd'-'z'] -> node(1)
        }
        node(1) {
            [Epsilon] -> node(2)
            [Epsilon] -> node(4)
        }
        node(2) {
            ['a'] -> node(3)
        }
        node(3) {
            [Epsilon] -> node(4)
            [Epsilon] -> node(2)
        }
        node(4) {}
        ")
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
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
    n0.connect(n2, Epsilon);
    n0.connect(n5, Epsilon);
    n2.connect(n3, b'a');
    n3.connect(n4, b'b');
    n4.connect(n1, Epsilon);
    n5.connect(n6, b'c');
    n6.connect(n7, b'd');
    n7.connect(n1, Epsilon);
    assert_eq!(
        dsp(&graph),
        dsp(&"
            node(0) {
                [Epsilon] -> node(2)
                [Epsilon] -> node(5)
            }
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                ['b'] -> node(4)
            }
            node(4) {
                [Epsilon] -> node(1)
            }
            node(1) {}
            node(5) {
                ['c'] -> node(6)
            }
            node(6) {
                ['d'] -> node(7)
            }
            node(7) {
                [Epsilon] -> node(1)
            }
        ")
    );
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_display_fmt_3() {
    let mut arena = Arena::new();
    let graph = Graph::dfa_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    a.connect(b, 1);
    b.connect(b, 3);
    b.connect(c, 1);
    assert_eq!(
        dbg(&graph),
        dsp(&"
            node(0) {
                [1] -> node(1)
            }
            node(1) {
                [3] -> self
                [1] -> node(2)
            }
            node(2) {}
        ")
    );
}
