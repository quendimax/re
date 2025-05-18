use pretty_assertions::{assert_eq, assert_ne};
use regr::{Epsilon, Graph, NodeId};
use std::collections::BTreeSet;

#[test]
fn node_id() {
    let graph_0 = Graph::dfa();
    let a = graph_0.node();
    let b = graph_0.node();

    assert_eq!(a.nid(), 0);
    assert_eq!(a.uid(), (a.gid() as u64) << NodeId::BITS);

    assert_eq!(b.nid(), 1);
    assert_eq!(b.uid(), ((b.gid() as u64) << NodeId::BITS) | 1);

    let graph_1 = Graph::dfa();
    let c = graph_1.node();
    let d = graph_1.node();

    assert_eq!(c.nid(), 0);
    assert_eq!(c.gid(), a.gid() + 1);
    assert_eq!(c.uid(), (c.gid() as u64) << NodeId::BITS);

    assert_eq!(d.nid(), 1);
    assert_eq!(d.gid(), a.gid() + 1);
    assert_eq!(d.uid(), ((c.gid() as u64) << NodeId::BITS) | 1);
}

#[test]
fn node_partial_eq() {
    let graph = Graph::nfa();
    let node_1 = graph.node();
    assert_ne!(node_1, graph.node());

    let graph_2 = Graph::nfa();
    let node_2 = graph_2.node();
    assert_ne!(node_1, node_2);
}

#[test]
fn node_connect() {
    let graph = Graph::nfa();
    let node_a = graph.node();
    let node_b = graph.node();
    let node_c = graph.node();
    node_a.connect(node_b, b'a');
    node_a.connect(node_c, b'a');
    node_a.connect(node_c, b'a');
    node_c.connect(node_a, Epsilon);
}

#[test]
fn node_closure() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();
    let e = graph.node();

    a.connect(b, b'a');
    b.connect(c, b'a');
    c.connect(d, Epsilon);
    b.connect(a, Epsilon);
    a.connect(d, Epsilon);
    d.connect(e, b'a');

    #[allow(clippy::mutable_key_type)]
    let set = BTreeSet::from_iter(vec![a, b, d, e]);
    assert_eq!(a.closure(b'a'), set)
}

#[test]
fn node_eclosure() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, b'a'..=u8::MAX);
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    c.connect(d, b'c');
    b.connect(a, Epsilon);
    d.connect(a, Epsilon);
    d.connect(b, Epsilon);
    d.connect(c, Epsilon);

    #[allow(clippy::mutable_key_type)]
    let set = BTreeSet::from_iter(vec![a, b, c]);
    assert_eq!(a.closure(Epsilon), set)
}

#[test]
fn node_symbol_targets() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, b'a'..=u8::MAX);
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    c.connect(d, b'c');
    b.connect(a, Epsilon);
    d.connect(a, Epsilon);
    d.connect(b, Epsilon);
    d.connect(c, Epsilon);

    assert_eq!(a.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![b]);
    assert_eq!(c.symbol_targets().map(|x| x.0).collect::<Vec<_>>(), vec![d]);
}

#[test]
#[should_panic]
fn node_symbol_targets_panic() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    a.connect(b, b'c');

    // expected that _node_tr is (Node, TransitionRef), and it locks writing to node a
    let _node_tr = a.symbol_targets().next();
    a.connect(b, b'a');
}

#[test]
fn node_epsilon_targets() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, b'a'..=u8::MAX);
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    c.connect(d, b'c');
    b.connect(a, Epsilon);
    d.connect(a, Epsilon);
    d.connect(b, Epsilon);
    d.connect(c, Epsilon);

    assert_eq!(a.epsilon_targets().collect::<Vec<_>>(), vec![b]);
}

#[test]
#[should_panic]
fn node_epsilon_targets_panics() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    a.connect(b, Epsilon);
    b.connect(c, Epsilon);
    for b in a.epsilon_targets() {
        for c in b.epsilon_targets() {
            b.connect(c, Epsilon);
        }
    }
}

#[test]
fn node_acceptize() {
    let graph = Graph::nfa();
    let a = graph.node();
    assert_eq!(format!("{:?}", a), "node(0)");
    a.acceptize();
    assert_eq!(format!("{:?}", a), "node((0))");
    a.disacceptize();
    assert_eq!(format!("{:?}", a), "node(0)");
}

#[test]
fn node_fmt_debug() {
    let graph = Graph::nfa();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node().acceptize();
    assert_eq!(format!("{:?}", a), "node(0)");
    assert_eq!(format!("{:?}", b), "node(1)");
    assert_eq!(format!("{:?}", c), "node((2))");
}
