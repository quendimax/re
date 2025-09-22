use pretty_assertions::assert_eq;
use redt::lit;
use regr::{Arena, Graph, Translator};
use resy::{Parser, enc::Utf8Encoder};

fn parse(pattern: &str) -> String {
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let parser = Parser::new(Utf8Encoder);
    let hir = parser.parse(pattern).unwrap();
    let mut translator = Translator::new(&graph);
    let start_node = graph.start_node();
    let end_node = graph.node();
    translator.translate(&hir, start_node, end_node);
    graph.to_string()
}

#[test]
#[should_panic]
fn translator_new_fails() {
    let mut arena = Arena::new();
    let graph = Graph::dfa_in(&mut arena);
    let _ = Translator::new(&graph);
}

#[test]
fn translate_literal() {
    assert_eq!(
        parse("sun"),
        lit!(
            ///node(0) {
            ///    ['s'] -> node(2)
            ///}
            ///node(2) {
            ///    ['u'] -> node(3)
            ///}
            ///node(3) {
            ///    ['n'] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}

#[test]
fn translate_class() {
    assert_eq!(
        parse("[a-ce]"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(4)
            ///}
            ///node(2) {
            ///    ['a'-'c'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
            ///node(4) {
            ///    ['e'] -> node(5)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
    assert_eq!(
        parse("[a-я]"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(4)
            ///    [Epsilon] -> node(7)
            ///}
            ///node(2) {
            ///    ['a'-7Fh] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
            ///node(4) {
            ///    [C2h-D0h] -> node(6)
            ///}
            ///node(6) {
            ///    [80h-BFh] -> node(5)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(7) {
            ///    [D1h] -> node(9)
            ///}
            ///node(9) {
            ///    [80h-8Fh] -> node(8)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
fn translate_group() {
    assert_eq!(
        parse("(?<1>)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///        wrpos r0
            ///}
            ///node(2) {
            ///    [Epsilon] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}
