#[cfg(not(feature = "ordered-hash"))]
pub type HashMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "ordered-hash"))]
pub type HashSet<T> = std::collections::HashSet<T>;

// LinkedHashMap preserves insertion order of key/value pairs. It needs only to
// get better legibility of a printed graph.
#[cfg(feature = "ordered-hash")]
pub type HashMap<K, V> = linked_hash_map::LinkedHashMap<K, V>;
#[cfg(feature = "ordered-hash")]
pub type HashSet<T> = linked_hash_set::LinkedHashSet<T>;
