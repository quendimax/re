//! LinkedHashMap/Set preserves insertion order of keys. It needs only to get
//! better legibility of printed graphs. Also it will allow to run some
//! benchmarks with different algorithms inside the sets and maps.

cfg_if::cfg_if! {
    if #[cfg(feature = "ordered-hash-map")] {
        pub type Map<K, V> = linked_hash_map::LinkedHashMap<K, V>;
        pub type MapIter<'a, K, V> = linked_hash_map::Iter<'a, K, V>;
    } else if #[cfg(feature = "hash-map")] {
        pub type Map<K, V> = std::collections::HashMap<K, V>;
        pub type MapIter<'a, K, V> = std::collections::hash_map::Iter<'a, K, V>;
    } else {
        pub type Map<K, V> = std::collections::BTreeMap<K, V>;
        pub type MapIter<'a, K, V> = std::collections::btree_map::Iter<'a, K, V>;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ordered-hash-set")] {
        pub type Set<T> = linked_hash_set::LinkedHashSet<T>;
        pub type SetIter<'a, T> = linked_hash_set::Iter<'a, T>;
    } else if #[cfg(feature = "hash-set")] {
        pub type Set<T> = std::collections::HashSet<T>;
        pub type SetIter<'a, T> = std::collections::hash_set::Iter<'a, T>;
    } else {
        pub type Set<T> = std::collections::BTreeSet<T>;
        pub type SetIter<'a, T> = std::collections::btree_set::Iter<'a, T>;
    }
}
