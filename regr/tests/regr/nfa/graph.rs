use pretty_assertions::assert_eq;
use regr::nfa::Graph;

#[test]
fn graph_new_with_capacity() {
    _ = Graph::<usize>::new();
    _ = Graph::<i8>::with_capacity(150);
}

#[test]
fn graph_node() {
    let graph = Graph::<char>::new();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
    let graph = Graph::<u32>::with_capacity(9);
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
}

#[test]
fn graph_start_node() {
    let graph = Graph::<char>::new();
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);

    let graph = Graph::<char>::new();
    assert_eq!(graph.node(), graph.start_node());
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.node().id(), 2);
    assert_eq!(graph.start_node().id(), 0);
}

// #[test]
// #[cfg_attr(any(feature = "hash-map", feature = "ordered-hash-map"), ignore)]
// fn node_print_graph_0() {
//     let arena = Graph::new();
//     let a = arena.node();
//     let b = arena.node();
//     let c = arena.node();
//     let d = arena.node();

//     a.connect(b, edge!['a'..=char::MAX]);
//     a.connect_with_epsilon(b);
//     b.connect_with_epsilon(c);
//     c.connect(d, edge!['c']);
//     b.connect_with_epsilon(a);
//     d.connect_with_epsilon(a);
//     d.connect_with_epsilon(b);
//     d.connect_with_epsilon(c);
//     let mut string = String::new();
//     a.print_graph(&mut string, "        ").unwrap();
//     assert_eq!(
//         string,
//         "        \
//         start 0:
//             ['a'-'\\u{10ffff}'] -> 1
//             EPSILON -> 1
//         node 1:
//             EPSILON -> 0
//             EPSILON -> 2
//         node 2:
//             ['c'] -> 3
//         node 3:
//             EPSILON -> 0
//             EPSILON -> 1
//             EPSILON -> 2"
//     );
// }
