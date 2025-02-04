use pretty_assertions::assert_eq;
use regr::Arena;

#[test]
fn arena_new() {
    _ = Arena::<usize>::new();
    _ = Arena::<i8>::with_capacity(150);
}

#[test]
fn arena_node_nfa() {
    let arena = Arena::<char>::new();
    assert_eq!(arena.node_nfa().id(), 0);
    assert_eq!(arena.node_nfa().id(), 1);
    assert_eq!(arena.node_nfa().id(), 2);
    let arena = Arena::<u32>::with_capacity(9);
    assert_eq!(arena.node_nfa().id(), 0);
    assert_eq!(arena.node_nfa().id(), 1);
    assert_eq!(arena.node_nfa().id(), 2);
}
