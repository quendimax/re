use ntest::assert_panics;
use pretty_assertions::{assert_eq, assert_str_eq};
use redt::SetU8;
use resy::Hir;

#[test]
fn hir_literal() {
    let lit = Hir::literal(b"hello");
    assert!(lit.is_literal());
    assert!(!lit.is_group());
    assert_eq!(lit.len_hint(), (5, Some(5)));
    assert_eq!(lit.exact_len(), Some(5));
    assert_str_eq!(lit.to_string(), "\"hello\"");

    let lit = Hir::literal(b"h");
    assert!(lit.is_literal());
    assert!(!lit.is_repeat());
    assert_eq!(lit.len_hint(), (1, Some(1)));
    assert_eq!(lit.exact_len(), Some(1));
    assert_str_eq!(lit.to_string(), "\"h\"");
}

#[test]
fn hir_class() {
    let mut set = SetU8::new();
    set.merge_byte(0);
    set.merge_range((27..=39).into());
    let class = Hir::class(set);
    assert!(class.is_class());
    assert!(!class.is_literal());
    assert_eq!(class.len_hint(), (1, Some(1)));
    assert_eq!(class.exact_len(), Some(1));
    assert_str_eq!(class.to_string(), r"[00h | 1Bh-'\'']");
}

#[test]
fn hir_group() {
    let lit = Hir::literal(b"hello");
    let group = Hir::group(2, lit);
    assert!(group.is_group());
    assert!(!group.is_class());
    assert_eq!(group.len_hint(), (5, Some(5)));
    assert_eq!(group.exact_len(), Some(5));
    assert_str_eq!(group.to_string(), r#"(?<2> "hello" )"#);
}

#[test]
fn hir_repeat() {
    let lit = Hir::literal(b"a");
    let repeat = Hir::repeat(lit, 0, None);
    assert!(repeat.is_repeat());
    assert!(!repeat.is_disjunct());
    assert_eq!(repeat.len_hint(), (0, None));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#""a"*"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (0, None));

    let lit = Hir::literal(b"abc");
    let repeat = Hir::repeat(lit, 1, None);
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (3, None));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#""abc"+"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (1, None));

    let class = Hir::class(SetU8::from(&[49, 50, 52]));
    let repeat = Hir::repeat(class, 0, Some(1));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (0, Some(1)));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"['1'-'2' | '4']?"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (0, Some(1)));

    let concat = Hir::concat(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(concat, 3, Some(3));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (15, Some(15)));
    assert_eq!(repeat.exact_len(), Some(15));
    assert_str_eq!(repeat.to_string(), r#"("ab" & "cde"){3}"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (3, Some(3)));

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(disjunct, 3, Some(5));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (6, Some(15)));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"("ab" | "cde"){3,5}"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (3, Some(5)));

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(disjunct, 3, None);
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (6, None));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"("ab" | "cde"){3,}"#);
    assert_eq!(repeat.unwrap_repeat().iter_hint(), (3, None));

    assert_panics!({
        let lit = Hir::literal(b"a");
        let _ = Hir::repeat(lit, 3, Some(2));
    });
}

#[test]
fn hir_concat() {
    let concat = Hir::concat(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    assert!(concat.is_concat());
    assert!(!concat.is_class());
    assert_eq!(concat.len_hint(), (5, Some(5)));
    assert_eq!(concat.exact_len(), Some(5));
    assert_str_eq!(concat.to_string(), r#""ab" & "cde""#);

    let lit = Hir::literal(b"abc");
    let repeat = Hir::repeat(lit, 1, None);
    let concat = Hir::concat(vec![Hir::literal(b"ab"), repeat]);
    assert!(concat.is_concat());
    assert_eq!(concat.len_hint(), (5, None));
    assert_eq!(concat.exact_len(), None);
    assert_str_eq!(concat.to_string(), r#""ab" & "abc"+"#);

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let concat = Hir::concat(vec![Hir::literal(b"ab"), disjunct]);
    assert!(concat.is_concat());
    assert_eq!(concat.len_hint(), (4, Some(5)));
    assert_eq!(concat.exact_len(), None);
    assert_str_eq!(concat.to_string(), r#""ab" & ("ab" | "cde")"#);
}

#[test]
fn hir_disjunct() {
    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    assert!(disjunct.is_disjunct());
    assert!(!disjunct.is_concat());
    assert_eq!(disjunct.len_hint(), (2, Some(3)));
    assert_eq!(disjunct.exact_len(), None);
    assert_str_eq!(disjunct.to_string(), r#""ab" | "cde""#);

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cd")]);
    assert!(disjunct.is_disjunct());
    assert_eq!(disjunct.len_hint(), (2, Some(2)));
    assert_eq!(disjunct.exact_len(), Some(2));
    assert_str_eq!(disjunct.to_string(), r#""ab" | "cd""#);

    let concat = Hir::concat(vec![
        Hir::literal(b"ab"),
        Hir::repeat(Hir::literal(b"cde"), 0, None),
    ]);
    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), concat]);
    assert!(disjunct.is_disjunct());
    assert_eq!(disjunct.len_hint(), (2, None));
    assert_eq!(disjunct.exact_len(), None);
    assert_str_eq!(disjunct.to_string(), r#""ab" | ("ab" & "cde"*)"#);
}

#[test]
fn hir_unwrap() {
    let lit = Hir::literal(b"a");
    assert_eq!(lit.unwrap_literal(), b"a");

    let class = Hir::class(Default::default());
    assert_eq!(class.unwrap_class(), Default::default());

    let repeat = Hir::group(1, Hir::literal(b"a"));
    _ = repeat.unwrap_group();

    let repeat = Hir::repeat(Hir::literal(b"a"), 0, None);
    _ = repeat.unwrap_repeat();

    let concat = Hir::concat(vec![]);
    _ = concat.unwrap_concat();

    let disjunct = Hir::disjunct(vec![]);
    _ = disjunct.unwrap_disjunct();

    assert_panics!({
        let lit = Hir::literal(b"a");
        _ = lit.unwrap_class();
    });

    assert_panics!({
        let class = Hir::class(Default::default());
        _ = class.unwrap_group();
    });

    assert_panics!({
        let class = Hir::group(1, Hir::literal(b"a"));
        _ = class.unwrap_repeat();
    });

    assert_panics!({
        let repeat = Hir::repeat(Hir::literal(b"a"), 0, None);
        _ = repeat.unwrap_concat();
    });

    assert_panics!({
        let concat = Hir::concat(vec![]);
        _ = concat.unwrap_disjunct();
    });

    assert_panics!({
        let disjunct = Hir::disjunct(vec![]);
        _ = disjunct.unwrap_literal();
    });
}
