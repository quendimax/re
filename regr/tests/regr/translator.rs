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
            ///node(1) {}
            ///node(2) {
            ///    ['u'] -> node(3)
            ///}
            ///node(3) {
            ///    ['n'] -> node(1)
            ///}
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
            ///node(1) {}
            ///node(2) {
            ///    ['a'-'c'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
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
            ///node(1) {}
            ///node(2) {
            ///    ['a'-7Fh] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(4) {
            ///    [C2h-D0h] -> node(6)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(6) {
            ///    [80h-BFh] -> node(5)
            ///}
            ///node(7) {
            ///    [D1h] -> node(9)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(9) {
            ///    [80h-8Fh] -> node(8)
            ///}
        )
    );
}

#[test]
fn translate_group_0() {
    assert_eq!(
        parse("(?<1>)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
    assert_eq!(
        parse("(?<1>a)b(?<2>c)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    ['b'] -> node(5)
            ///}
            ///node(3) {
            ///    ['a'] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(6)
            ///}
            ///node(6) {
            ///    ['c'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
    assert_eq!(
        parse("(?<1>)(?<2>)(?<234>)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(6)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(8)
            ///}
            ///node(6) {
            ///    [Epsilon] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(9)
            ///}
            ///node(9) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
fn translate_group_1() {
    assert_eq!(
        parse("(?<1>)(a|bc)(?<2>)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(6)
            ///    [Epsilon] -> node(8)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(11)
            ///        wrpos t5/r1
            ///}
            ///node(6) {
            ///    ['a'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(8) {
            ///    ['b'] -> node(10)
            ///}
            ///node(9) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(10) {
            ///    ['c'] -> node(9)
            ///}
            ///node(11) {
            ///    [Epsilon] -> node(12)
            ///}
            ///node(12) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
    assert_eq!(
        parse("(?<1>)(a|b)(?<2>)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(6)
            ///    [Epsilon] -> node(8)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(10)
            ///}
            ///node(6) {
            ///    ['a'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(8) {
            ///    ['b'] -> node(9)
            ///}
            ///node(9) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(10) {
            ///    [Epsilon] -> node(11)
            ///}
            ///node(11) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
fn translate_group_2() {
    assert_eq!(
        parse("(?<1>)((?<2>a)|b(?<3>a))"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(5)
            ///    [Epsilon] -> node(9)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(7)
            ///}
            ///node(6) {
            ///    [Epsilon] -> node(1)
            ///        invd t6
            ///}
            ///node(7) {
            ///    ['a'] -> node(8)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(6)
            ///}
            ///node(9) {
            ///    ['b'] -> node(11)
            ///}
            ///node(10) {
            ///    [Epsilon] -> node(1)
            ///        invd t3
            ///}
            ///node(11) {
            ///    [Epsilon] -> node(12)
            ///}
            ///node(12) {
            ///    ['a'] -> node(13)
            ///}
            ///node(13) {
            ///    [Epsilon] -> node(10)
            ///}
        )
    );
    assert_eq!(
        parse("(?<1>((?<2>a((?<3>d)|(?<5>e)))|b(?<4>a)))"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(4)
            ///    [Epsilon] -> node(17)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(6)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(3)
            ///        invd t11
            ///}
            ///node(6) {
            ///    ['a'] -> node(8)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(9)
            ///    [Epsilon] -> node(13)
            ///}
            ///node(9) {
            ///    [Epsilon] -> node(11)
            ///}
            ///node(10) {
            ///    [Epsilon] -> node(7)
            ///        invd t7
            ///}
            ///node(11) {
            ///    ['d'] -> node(12)
            ///}
            ///node(12) {
            ///    [Epsilon] -> node(10)
            ///}
            ///node(13) {
            ///    [Epsilon] -> node(15)
            ///}
            ///node(14) {
            ///    [Epsilon] -> node(7)
            ///        invd t4
            ///}
            ///node(15) {
            ///    ['e'] -> node(16)
            ///}
            ///node(16) {
            ///    [Epsilon] -> node(14)
            ///}
            ///node(17) {
            ///    ['b'] -> node(19)
            ///}
            ///node(18) {
            ///    [Epsilon] -> node(3)
            ///        invd t2
            ///        invd t4
            ///        invd t7
            ///}
            ///node(19) {
            ///    [Epsilon] -> node(20)
            ///}
            ///node(20) {
            ///    ['a'] -> node(21)
            ///}
            ///node(21) {
            ///    [Epsilon] -> node(18)
            ///}
        )
    );
}

#[test]
fn translate_group_3() {
    assert_eq!(
        parse("(?<1>a)(?<2>b)*(?<3>c)"),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(3)
            ///        wrpos t0/r0
            ///}
            ///node(1) {}
            ///node(2) {
            ///    [Epsilon] -> node(5)
            ///    [Epsilon] -> node(6)
            ///}
            ///node(3) {
            ///    ['a'] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(10)
            ///}
            ///node(6) {
            ///    [Epsilon] -> node(8)
            ///        wrpos t3/r1
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(5)
            ///    [Epsilon] -> node(6)
            ///}
            ///node(8) {
            ///    ['b'] -> node(9)
            ///}
            ///node(9) {
            ///    [Epsilon] -> node(7)
            ///}
            ///node(10) {
            ///    ['c'] -> node(11)
            ///}
            ///node(11) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}
