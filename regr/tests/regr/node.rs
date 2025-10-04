use pretty_assertions::{assert_eq, assert_ne};
use redt::range;
use regr::{Arena, Epsilon, Graph};
use std::collections::BTreeSet;

#[test]
fn node_copy_and_clone() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let node = graph.node();
    #[allow(clippy::clone_on_copy)]
    let cloned_node = node.clone();
    let copied_node = node;
    assert_eq!(node.nid(), cloned_node.nid());
    assert_eq!(node.gid(), cloned_node.gid());
    assert_eq!(node.uid(), cloned_node.uid());
    assert_eq!(node.nid(), copied_node.nid());
    assert_eq!(node.gid(), copied_node.gid());
    assert_eq!(node.uid(), copied_node.uid());
}

#[test]
fn node_id() {
    let mut arena_0 = Arena::new();
    let graph_0 = Graph::new_in(&mut arena_0);
    let a = graph_0.node();
    let b = graph_0.node();

    assert_eq!(a.nid(), 0);
    assert_eq!(a.uid(), (a.gid() as u64) << (u64::BITS / 2));

    assert_eq!(b.nid(), 1);
    assert_eq!(b.uid(), ((b.gid() as u64) << (u64::BITS / 2)) | 1);

    let mut arena_1 = Arena::new();
    let graph_1 = Graph::new_in(&mut arena_1);
    let c = graph_1.node();
    let d = graph_1.node();

    assert_eq!(c.nid(), 0);
    assert_ne!(c.gid(), a.gid());
    assert_eq!(c.uid(), (c.gid() as u64) << (u64::BITS / 2));

    assert_eq!(d.nid(), 1);
    assert_ne!(d.gid(), a.gid());
    assert_eq!(d.uid(), ((c.gid() as u64) << (u64::BITS / 2)) | 1);
}

#[test]
fn node_partial_eq() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let node_1 = graph.node();
    assert_ne!(node_1, graph.node());
    drop(graph);

    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let node_2 = graph.node();
    assert_ne!(node_1, node_2);
}

#[test]
fn node_connect_nfa() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let node_a = graph.node();
    let node_b = graph.node();
    let node_c = graph.node();
    node_a.connect(node_b).merge(b'a');
    node_a.connect(node_c).merge(b'a');
    node_a.connect(node_c).merge(b'a');
    node_c.connect(node_a);
}

#[test]
fn node_connect_dfa() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let node_a = graph.node();
    let node_b = graph.node();
    node_a.connect(node_b).merge(b'a');
}

#[test]
#[should_panic(expected = "only nodes of the same graph can be joint")]
fn node_connect_panics() {
    let mut arena_a = Arena::new();
    let mut arena_b = Arena::new();
    let graph_a = Graph::new_in(&mut arena_a);
    let graph_b = Graph::new_in(&mut arena_b);
    let node_a = graph_a.node();
    let node_b = graph_b.node();
    node_a.connect(node_b);
}

#[test]
fn node_closure() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();
    let e = graph.node();

    a.connect(b).merge(b'a');
    b.connect(c).merge(b'a');
    c.connect(d);
    b.connect(a);
    a.connect(d);
    d.connect(e).merge(b'a');

    #[allow(clippy::mutable_key_type)]
    let set = BTreeSet::from_iter(vec![a, b, d, e]);
    assert_eq!(a.closure(b'a'), set)
}

#[test]
fn node_eclosure() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b);
    b.connect(c);
    c.connect(d).merge(b'c');
    b.connect(a);
    d.connect(a);
    d.connect(b);
    d.connect(c);

    #[allow(clippy::mutable_key_type)]
    let set = BTreeSet::from_iter(vec![a, b, c]);
    assert_eq!(a.closure(Epsilon), set)
}

#[test]
fn node_symbol_targets() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b).merge(range(b'a', u8::MAX));
    a.connect(b);
    b.connect(c);
    c.connect(d).merge(b'c');
    b.connect(a);
    d.connect(a);
    d.connect(b);
    d.connect(c);

    assert_eq!(a.targets().keys().copied().collect::<Vec<_>>(), vec![b]);
    assert_eq!(c.targets().keys().copied().collect::<Vec<_>>(), vec![d]);
}

#[test]
#[should_panic]
fn node_symbol_targets_panic() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    a.connect(b).merge(b'c');

    // expected that _node_tr is (Node, Transition), and it locks writing to node a
    for _ in a.targets().iter() {
        a.connect(b).merge(b'a');
    }
}

#[test]
fn node_collect_epsilon_targets() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b);
    b.connect(c);
    c.connect(d).merge(b'c');
    b.connect(a);
    d.connect(a);
    d.connect(b);
    d.connect(c);

    assert_eq!(a.collect_epsilon_targets::<Vec<_>>(), vec![b]);
    assert_eq!(c.collect_epsilon_targets::<Vec<_>>(), vec![]);
}

#[test]
#[should_panic]
fn node_iter_and_connect_overlap_panics() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    a.connect(b);
    b.connect(c);
    a.for_each_epsilon_target(|b| {
        b.for_each_epsilon_target(|c| {
            b.connect(c);
        });
    });
}

#[test]
fn node_finalize() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    assert_eq!(format!("{a:?}"), "node(0)");
    a.finalize();
    assert_eq!(format!("{a:?}"), "node((0))");
    a.definalize();
    assert_eq!(format!("{a:?}"), "node(0)");
}

#[test]
fn node_fmt_debug() {
    let mut arena = Arena::new();
    let graph = Graph::new_in(&mut arena);
    let a = graph.node();
    let b = graph.node();
    let c = graph.node().finalize();
    assert_eq!(format!("{a:?}"), "node(0)");
    assert_eq!(format!("{b:?}"), "node(1)");
    assert_eq!(format!("{c:?}"), "node((2))");
}
