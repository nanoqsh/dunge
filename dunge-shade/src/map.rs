use {
    ahash::AHasher,
    std::{collections::HashMap, hash::BuildHasherDefault},
};

pub(crate) type Map<K, V> = HashMap<K, V, BuildHasherDefault<AHasher>>;

pub(crate) const fn make<K, V>() -> Map<K, V> {
    Map::with_hasher(BuildHasherDefault::new())
}
