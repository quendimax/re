use pretty_assertions::assert_eq;
use regex_syntax::Parser;
use regr::nfa;
use regr::{err, Result};
use regr::{Arena, Translator};

type Node<'a> = nfa::Node<'a, u8>;

fn translate<'a, 'b>(pattern: &'a str, arena: &'b Arena<u8>) -> Result<(Node<'b>, Node<'b>)> {
    let translator = Translator::new(&arena);
    let mut parser = Parser::new();
    let hir = parser.parse(pattern).unwrap();
    translator.from_hir_to_nfa(&hir)
}

#[test]
fn translate_alternation() {
    let arena = Arena::new();
    let (start, finish) = translate("abc|df", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);
}

#[test]
fn translate_capture() {
    let arena = Arena::new();
    let (start, finish) = translate("(abc)", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 3);

    let (start, finish) = translate("(abc)|(?:cd)", &arena).unwrap();
    assert_eq!(start.id(), 4);
    assert_eq!(finish.id(), 5);
}

#[test]
fn translate_class() {
    let arena = Arena::new();
    let (start, finish) = translate(r"[a-b]", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);
}

#[test]
fn translate_concat() {
    let arena = Arena::new();
    let (start, finish) = translate(r"abc(df)", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 5);
}

#[test]
fn translate_empty() {
    let arena = Arena::new();
    let (start, finish) = translate("", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);

    let (start, finish) = translate("a|", &arena).unwrap();
    assert_eq!(start.id(), 2);
    assert_eq!(finish.id(), 3);
}

#[test]
fn translate_literal() {
    let arena = Arena::new();
    let (start, finish) = translate("abў", &arena).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 4);

    let (start, finish) = translate(r"a\x64", &arena).unwrap();
    assert_eq!(start.id(), 5);
    assert_eq!(finish.id(), 7);
}

#[test]
fn translate_look() {
    let arena = Arena::new();
    assert_eq!(
        translate(r"\b", &arena),
        err::unsupported_feature("look around is not supported")
    );
}

#[test]
fn translate_repetition() {
    let arena = Arena::new();
    assert_eq!(
        translate("a*?", &arena),
        err::unsupported_feature("non-greedy repetitions are not supported")
    );

    let (start, finish) = translate("a*", &arena).unwrap();
    assert_eq!(start.id(), 1);
    assert_eq!(finish.id(), 4);

    let (start, finish) = translate("a+", &arena).unwrap();
    assert_eq!(start.id(), 5);
    assert_eq!(finish.id(), 9);

    let (start, finish) = translate("a{3,}", &arena).unwrap();
    assert_eq!(start.id(), 10);
    assert_eq!(finish.id(), 16);

    let (start, finish) = translate("a{3,5}", &arena).unwrap();
    assert_eq!(start.id(), 17);
    assert_eq!(finish.id(), 22);

    let (start, finish) = translate("a{3}", &arena).unwrap();
    assert_eq!(start.id(), 23);
    assert_eq!(finish.id(), 26);
}
