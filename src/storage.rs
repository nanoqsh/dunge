use {crate::r#loop::Error, ahash::AHashMap as Map};

pub(crate) struct Storage<T> {
    map: Map<u32, T>,
    counter: u32,
}

impl<T> Storage<T> {
    pub(crate) fn insert(&mut self, value: T) -> u32 {
        let index = self.counter;
        self.counter = self.counter.wrapping_add(1);
        self.map.insert(index, value);
        index
    }

    pub(crate) fn get(&self, index: u32) -> Result<&T, Error> {
        self.map.get(&index).ok_or(Error::ResourceNotFound)
    }

    pub(crate) fn get_mut(&mut self, index: u32) -> Result<&mut T, Error> {
        self.map.get_mut(&index).ok_or(Error::ResourceNotFound)
    }

    pub(crate) fn remove(&mut self, index: u32) -> Result<(), Error> {
        self.map
            .remove(&index)
            .map(drop)
            .ok_or(Error::ResourceNotFound)
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
