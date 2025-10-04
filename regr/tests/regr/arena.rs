use pretty_assertions::{assert_eq, assert_ne};
use redt::lit;
use regr::{Arena, Graph};

#[test]
fn arena_ctor() {
    _ = Arena::new();
    _ = Arena::default();
    _ = Arena::with_capacity(10);
}

#[test]
fn arena_alloc_node() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    // ERROR: mutable borrow: _ = arena.alloc_node();
    _ = gr.node();
}

#[test]
fn arena_binds_graph() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let first_gid = gr.gid();
    drop(gr);

    let gr = Graph::new_in(&mut arena);
    let second_gid = gr.gid();
    assert_ne!(first_gid, second_gid);
}

#[test]
fn arena_alloc_during_iteration() {
    let mut arena = Arena::with_capacity(0);
    let gr = Graph::new_in(&mut arena);
    let mut items = Vec::new();
    for _ in 0..5 {
        let node = gr.node();
        items.push(node.nid());
    }

    let arena = gr.arena();

    for _ in arena.nodes() {
        let node = gr.node();
        items.push(node.nid());
    }

    assert_eq!(items.len(), 10);
    assert_eq!(arena.nodes().len(), 10);
    assert_eq!(
        items,
        arena.nodes().map(|node| node.nid()).collect::<Vec<_>>()
    );
}

#[test]
fn arena_fmt_display() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let a = gr.node();
    a.connect(a);
    let b = gr.node();
    a.connect(b).merge(b'a');

    assert_eq!(
        format!("{}", gr.arena()),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> self
            ///    ['a'] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}
