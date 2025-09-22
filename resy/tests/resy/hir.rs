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

    let empty = Hir::empty();
    assert!(empty.is_literal());
    assert!(!empty.is_repeat());
    assert_eq!(empty.len_hint(), (0, Some(0)));
    assert_eq!(empty.exact_len(), Some(0));
    assert_str_eq!(empty.to_string(), "\"\"");
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
    if let Hir::Group(hir) = group {
        assert_eq!(hir.inner(), &Hir::literal("hello"));
    }
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
    if let Hir::Repeat(hir) = repeat {
        assert_eq!(hir.inner(), &Hir::literal("a"));
        assert_eq!(hir.iter_hint(), (0, None));
    }

    let lit = Hir::literal(b"abc");
    let repeat = Hir::repeat(lit, 1, None);
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (3, None));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#""abc"+"#);

    let class = Hir::class(SetU8::from(&[49, 50, 52]));
    let repeat = Hir::repeat(class, 0, Some(1));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (0, Some(1)));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"['1'-'2' | '4']?"#);

    let concat = Hir::concat(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(concat, 3, Some(3));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (15, Some(15)));
    assert_eq!(repeat.exact_len(), Some(15));
    assert_str_eq!(repeat.to_string(), r#"("ab" & "cde"){3}"#);

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(disjunct, 3, Some(5));
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (6, Some(15)));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"("ab" | "cde"){3,5}"#);

    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), Hir::literal(b"cde")]);
    let repeat = Hir::repeat(disjunct, 3, None);
    assert!(repeat.is_repeat());
    assert_eq!(repeat.len_hint(), (6, None));
    assert_eq!(repeat.exact_len(), None);
    assert_str_eq!(repeat.to_string(), r#"("ab" | "cde"){3,}"#);

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
    if let Hir::Concat(hir) = concat {
        assert_eq!(hir.items().len(), 2);
    }

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
    if let Hir::Disjunct(hir) = disjunct {
        assert_eq!(hir.alternatives().len(), 2);
        assert_eq!(hir.exact_len(), Some(2));
    }

    let concat = Hir::concat(vec![
        Hir::literal(b"ab"),
        Hir::repeat(Hir::literal(b"cde"), 0, None),
    ]);
    let disjunct = Hir::disjunct(vec![Hir::literal(b"ab"), concat]);
    assert!(disjunct.is_disjunct());
    assert_eq!(disjunct.len_hint(), (2, None));
    assert_eq!(disjunct.exact_len(), None);
    assert_str_eq!(disjunct.to_string(), r#""ab" | ("ab" & "cde"*)"#);
    if let Hir::Disjunct(hir) = disjunct {
        assert_eq!(hir.alternatives().len(), 2);
        assert_eq!(hir.exact_len(), None);
    }
}

#[test]
#[should_panic(expected = "empty disjunction is not allowed")]
fn hir_disjunct_fails() {
    let _ = Hir::disjunct(vec![]);
}
