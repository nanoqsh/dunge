use {crate::error::ResourceNotFound, ahash::AHashMap as Map};

pub(crate) struct Storage<T> {
    map: Map<u32, T>,
    counter: u32,
}

impl<T> Storage<T> {
    pub fn insert(&mut self, value: T) -> u32 {
        use std::collections::hash_map::Entry;

        loop {
            let index = self.counter;
            self.counter = self.counter.wrapping_add(1);

            if let Entry::Vacant(en) = self.map.entry(index) {
                en.insert(value);
                break index;
            }
        }
    }

    pub fn get(&self, index: u32) -> Result<&T, ResourceNotFound> {
        self.map.get(&index).ok_or(ResourceNotFound)
    }

    pub fn get_mut(&mut self, index: u32) -> Result<&mut T, ResourceNotFound> {
        self.map.get_mut(&index).ok_or(ResourceNotFound)
    }

    pub fn remove(&mut self, index: u32) -> Result<(), ResourceNotFound> {
        self.map.remove(&index).map(drop).ok_or(ResourceNotFound)
    }
}

impl<T> Default for Storage<T> {
    fn default() -> Self {
        Self {
            map: Map::default(),
            counter: 0,
        }
    }
}
