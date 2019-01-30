use chashmap::CHashMap;
use std::hash::Hash;
use std::sync::{Arc, Weak};

use crate::{Allocator, Id};

/// An allocator based on a concurrent hashmap
pub struct HashAllocator<T> {
    backing: CHashMap<Arc<T>, Weak<T>>,
}

impl<T> Default for HashAllocator<T> {
    fn default() -> Self {
        let backing = CHashMap::new();
        Self { backing }
    }
}

impl<T: Eq + Hash> Allocator<T> for HashAllocator<T> {
    fn allocate(&self, t: T) -> Id<T> {
        let key = Arc::new(t);
        let value = Arc::downgrade(&key);
        let mut result = Arc::clone(&key);

        self.backing.upsert(
            key,
            || value,
            |other| {
                result = Weak::upgrade(other).unwrap();
            },
        );
        Id(result)
    }

    fn allocations(&self) -> usize {
        self.backing.len()
    }

    fn delete_unused(&self) {
        // OK since each bucket is locked first
        self.backing
            .retain(|key, _value| Arc::strong_count(key) > 1);
        self.backing.shrink_to_fit();
    }
}
