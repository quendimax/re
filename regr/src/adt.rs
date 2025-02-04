#[cfg(not(any(feature = "hash-map", feature = "ordered-hash-map")))]
pub type Map<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(not(any(feature = "hash-set", feature = "ordered-hash-set")))]
pub type Set<T> = std::collections::BTreeSet<T>;

#[cfg(all(feature = "hash-map", not(feature = "ordered-hash-map")))]
pub type Map<K, V> = std::collections::HashMap<K, V>;
#[cfg(all(feature = "hash-set", not(feature = "ordered-hash-set")))]
pub type Set<T> = std::collections::HashSet<T>;

// LinkedHashMap preserves insertion order of key/value pairs. It needs only to
// get better legibility of printed graphs.
#[cfg(feature = "ordered-hash-map")]
pub type Map<K, V> = linked_hash_map::LinkedHashMap<K, V>;
#[cfg(feature = "ordered-hash-set")]
pub type Set<T> = linked_hash_set::LinkedHashSet<T>;
