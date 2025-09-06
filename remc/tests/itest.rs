use remc::re;

#[test]
fn simple_regex() {
    let mut regex = re!("hello");
    let m = regex.match_at("hello", 0).unwrap();
    assert_eq!(m.as_str(), "hello");

    let m = regex.match_at("hhelloo", 0);
    assert_eq!(m, None);

    let m = regex.match_at("hhelloo", 1).unwrap();
    assert_eq!(m.as_str(), "hello");
}

#[test]
fn klenee_start_regex() {
    let mut regex = re!("hello**");
    let m = regex.match_at("hello", 0).unwrap();
    assert_eq!(m.as_str(), "hello");

    let m = regex.match_at("hhelloo", 0);
    assert_eq!(m, None);

    let m = regex.match_at("hhelloooO", 1).unwrap();
    assert_eq!(m.as_str(), "hellooo");

    let mut regex = re!("hell[oa]**");
    let m = regex.match_at("hella", 0).unwrap();
    assert_eq!(m.as_str(), "hella");

    let m = regex.match_at("hell", 0).unwrap();
    assert_eq!(m.as_str(), "hell");

    let m = regex.match_at("hhellaoaO", 1).unwrap();
    assert_eq!(m.as_str(), "hellaoa");
    assert_eq!(m.as_bytes(), b"hellaoa");
}

#[test]
fn another_test() {
    let mut regex = re!("[ab]*a");
    let m = regex.match_at("ba", 0).unwrap();
    assert_eq!(m.as_str(), "ba");
    let m = regex.match_at("baaaaaa", 0).unwrap();
    assert_eq!(m.as_str(), "baaaaaa");
    let m = regex.match_at("aaaaab", 0).unwrap();
    assert_eq!(m.as_str(), "aaaaa");
    let m = regex.match_at("bbb", 0);
    assert_eq!(m, None);
}
