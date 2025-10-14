use pretty_assertions::assert_eq;
use redt::{map, set};

#[test]
fn set_macro() {
    let set = set! {
        "value1",
        "value2", // trailing comma
    };
    assert!(set.contains("value1"));
    assert!(set.contains("value2"));
    assert_eq!(set.len(), 2);

    let set = set! {
        1,
        1,
        3,
        2 // no comma
    };
    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
    assert_eq!(set.len(), 3);
}

#[test]
fn map_macro() {
    let map = map! {
        "key1" => "value1",
        "key2" => "value2",  // trailing comma
    };
    assert_eq!(map.get("key1"), Some(&"value1"));
    assert_eq!(map.get("key2"), Some(&"value2"));
    assert_eq!(map.len(), 2);

    let map = map! {
        1 => "value1",
        2 => "value2",  // no trailing comma
        2 => "value3",  // no trailing comma
    };
    assert_eq!(map.get(&1), Some(&"value1"));
    assert_eq!(map.get(&2), Some(&"value3"));
    assert_eq!(map.len(), 2);
}
