use pretty_assertions::assert_eq;
use recz::{Label, re};

#[test]
fn concat() {
    let re = re!("асюсяй");
    assert_eq!(re.pattern(), "асюсяй");
    assert_eq!(re.capture_labels(), [Label::Num(0)]);

    let m = re.mtch("асюсяйка").unwrap();
    assert_eq!(m.haystack(), "асюсяйка");
    assert_eq!(m.as_str(), "асюсяй");
    assert_eq!(m.range(), (0..12).into());
    assert_eq!(m.capture(0).unwrap().as_str(), "асюсяй");

    assert_eq!(re.mtch("асюсяюшка"), None);
}

#[test]
fn hello() {
    let re = re!("h[ae](?<tail>llo*)");
    assert_eq!(re.pattern(), "h[ae](?<tail>llo*)");
    assert_eq!(re.capture_labels(), [Label::Num(0), Label::Str("tail")]);

    let m = re.mtch("hello").unwrap();
    assert_eq!(m.haystack(), "hello");
    assert_eq!(m.capture(0).unwrap().as_str(), "hello");
    assert_eq!(m.capture("tail").unwrap().as_str(), "llo");

    let m = re.mtch("helloooooasdf").unwrap();
    assert_eq!(m.haystack(), "helloooooasdf");
    assert_eq!(m.capture(0).unwrap().as_str(), "hellooooo");
    assert_eq!(m.capture("tail").unwrap().as_str(), "llooooo");
}

#[test]
fn hello2() {
    let re = re!("h((?<left>a)|(?<right>e))(?<tail>llo*)");
    assert_eq!(re.pattern(), "h((?<left>a)|(?<right>e))(?<tail>llo*)");
    assert_eq!(
        re.capture_labels(),
        [
            Label::Num(0),
            Label::Str("left"),
            Label::Str("right"),
            Label::Str("tail")
        ]
    );

    let m = re.mtch("hello").unwrap();
    assert_eq!(m.haystack(), "hello");
    assert_eq!(m.capture(0).unwrap().as_str(), "hello");
    assert_eq!(m.capture("tail").unwrap().as_str(), "llo");
    assert_eq!(m.capture("left"), None);
    assert_eq!(m.capture("right").unwrap().as_str(), "e");

    let m = re.mtch("halloooooasdf").unwrap();
    assert_eq!(m.haystack(), "halloooooasdf");
    assert_eq!(m.capture(0).unwrap().as_str(), "hallooooo");
    assert_eq!(m.capture("tail").unwrap().as_str(), "llooooo");
    assert_eq!(m.capture("left").unwrap().as_str(), "a");
    assert_eq!(m.capture("right"), None);
}

#[test]
fn star_ambiguity_0() {
    let re = re!("(?<ab>[ab]*)(?<a>a*)");

    let m = re.mtch("aaa").unwrap();
    assert_eq!(m.haystack(), "aaa");
    assert_eq!(m.capture(0).unwrap().range(), (0..3).into());
    assert_eq!(m.capture("ab").unwrap().range(), (0..3).into());
    assert_eq!(m.capture("a").unwrap().range(), (3..3).into());

    let m = re.mtch("babaa").unwrap();
    assert_eq!(m.haystack(), "babaa");
    assert_eq!(m.capture(0).unwrap().range(), (0..5).into());
    assert_eq!(m.capture("ab").unwrap().range(), (0..5).into());
    assert_eq!(m.capture("a").unwrap().range(), (5..5).into());
}

#[test]
fn star_ambiguity_1() {
    let re = re!("(?<sup>(?<sub>a*)*)");

    let m = re.mtch("aaa").unwrap();
    assert_eq!(m.haystack(), "aaa");
    assert_eq!(m.capture(0).unwrap().range(), (0..3).into());
    assert_eq!(m.capture("sup").unwrap().range(), (0..3).into());
    assert_eq!(m.capture("sub").unwrap().range(), (3..3).into());
}
