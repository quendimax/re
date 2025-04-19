use pretty_assertions::assert_eq;
use regr::dfa::Graph;
use regr::edge;

#[test]
fn graph_ctor() {
    _ = Graph::new();
    _ = Graph::default();
    _ = Graph::with_capacity(8);
}

#[test]
fn graph_node() {
    let graph = Graph::new();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
}

#[test]
fn graph_start_node() {
    let graph = Graph::new();
    assert_eq!(graph.node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.start_node().id(), 0);

    let graph = Graph::new();
    assert_eq!(graph.start_node().id(), 0);
    assert_eq!(graph.node().id(), 1);
    assert_eq!(graph.start_node().id(), 0);
}

#[test]
#[cfg_attr(not(feature = "ordered-hash"), ignore)]
fn graph_debug_fmt()
{
    let graph = Graph::new();
    let a = graph.node();
    let b = graph.node();
    let c = graph.node();
    let d = graph.node();

    a.connect(b, edge![b'a'..=u8::MAX]);
    c.connect(d, edge![b'c']);
    a.connect(c, edge![3, 4, 5]);
    d.connect(a, edge![1..=123, 150..=151]);
    assert_eq!(
        format!("{:?}", graph).replace("\n", "\n        "),
        "\
        node 0 {
            [3-5] -> node 2
            ['a'-255] -> node 1
        }
        node 1 {}
        node 2 {
            ['c'] -> node 3
        }
        node 3 {
            [1-'{'] -> node 0
            [150-151] -> node 0
        }"
    );
}
