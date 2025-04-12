use pretty_assertions::assert_eq;
use regex_syntax::Parser;
use regr::Translator;
use regr::nfa::{Graph, Node};
use regr::{Result, err};

fn translate<'a, 'b>(pattern: &'a str, graph: &'b Graph) -> Result<(Node<'b>, Node<'b>)> {
    let translator = Translator::new(&graph);
    let mut parser = Parser::new();
    let hir = parser.parse(pattern).unwrap();
    translator.from_hir_to_nfa(&hir)
}

#[test]
fn translate_alternation() {
    let graph = Graph::new();
    let (start, finish) = translate("abc|df", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);
}

#[test]
fn translate_capture() {
    let graph = Graph::new();
    let (start, finish) = translate("(abc)", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 3);

    let (start, finish) = translate("(abc)|(?:cd)", &graph).unwrap();
    assert_eq!(start.id(), 4);
    assert_eq!(finish.id(), 5);
}

#[test]
fn translate_unicode_class() {
    let graph = Graph::new();
    let (start, finish) = translate(r"[a-bъ-ўऄ-ऩ𐂂-𐂅]", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);
}


#[test]
fn translate_byte_class() {
    let graph = Graph::new();
    let (start, finish) = translate(r"(?i-u)[a-b]", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);
}

#[test]
fn translate_concat() {
    let graph = Graph::new();
    let (start, finish) = translate(r"abc(df)", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 5);
}

#[test]
fn translate_empty() {
    let graph = Graph::new();
    let (start, finish) = translate("", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 1);

    let (start, finish) = translate("a|", &graph).unwrap();
    assert_eq!(start.id(), 2);
    assert_eq!(finish.id(), 3);
}

#[test]
fn translate_literal() {
    let graph = Graph::new();
    let (start, finish) = translate("abў", &graph).unwrap();
    assert_eq!(start.id(), 0);
    assert_eq!(finish.id(), 4);

    let (start, finish) = translate(r"a\x64", &graph).unwrap();
    assert_eq!(start.id(), 5);
    assert_eq!(finish.id(), 7);
}

#[test]
fn translate_look() {
    let graph = Graph::new();
    assert_eq!(
        translate(r"\b", &graph),
        err::unsupported_feature("look around is not supported")
    );
}

#[test]
fn translate_repetition() {
    let graph = Graph::new();
    assert_eq!(
        translate("a*?", &graph),
        err::unsupported_feature("non-greedy repetitions are not supported")
    );

    let (start, finish) = translate("a*", &graph).unwrap();
    assert_eq!(start.id(), 1);
    assert_eq!(finish.id(), 4);

    let (start, finish) = translate("a+", &graph).unwrap();
    assert_eq!(start.id(), 5);
    assert_eq!(finish.id(), 9);

    let (start, finish) = translate("a{3,}", &graph).unwrap();
    assert_eq!(start.id(), 10);
    assert_eq!(finish.id(), 16);

    let (start, finish) = translate("a{3,5}", &graph).unwrap();
    assert_eq!(start.id(), 17);
    assert_eq!(finish.id(), 22);

    let (start, finish) = translate("a{3}", &graph).unwrap();
    assert_eq!(start.id(), 23);
    assert_eq!(finish.id(), 26);
}
