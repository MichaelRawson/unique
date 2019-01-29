use chashmap::CHashMap;
use std::hash::{Hash, Hasher};
use std::mem::drop;
use std::ptr::NonNull;

use crate::Id;

struct Key<T>(*const T);

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0)
    }
}

impl<T> Copy for Key<T> {}
unsafe impl<T> Sync for Key<T> {}
unsafe impl<T> Send for Key<T> {}

impl<T: PartialEq> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.0).eq(&*other.0) }
    }
}

impl<T: Hash> Hash for Key<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        unsafe {
            (*self.0).hash(hasher);
        }
    }
}

/// A backing store based on a concurrent hashmap.
pub struct HashBacking<T> {
    backing: CHashMap<Key<T>, Id<T>>,
}

impl<T> HashBacking<T> {
    /// How many items are currently stored?
    pub fn num_entries(&self) -> usize {
        self.backing.len()
    }
}

impl<T> HashBacking<T> {
    /// Create a new backing store, pre-allocating `capacity` items.
    pub fn new(capacity: usize) -> Self {
        HashBacking {
            backing: CHashMap::with_capacity(capacity),
        }
    }
}

impl<T: PartialEq + Hash> HashBacking<T> {
    /// Allows implementing `Backed` for any type that implements `Eq + Hash`.
    pub fn unique(&self, val: T) -> Id<T> {
        let key = Key(&val);
        if let Some(id) = self.backing.get(&key) {
            return *id;
        }

        let boxed = Box::new(val);
        let pointer = Box::into_raw(boxed);
        let key = Key(pointer);
        let id = Id(unsafe { NonNull::new_unchecked(pointer) });
        let mut insert_failed = false;

        self.backing.upsert(
            key,
            || id,
            |_| {
                insert_failed = true;
            },
        );

        let result = *self.backing.get(&key).unwrap();
        if insert_failed {
            let reboxed = unsafe { Box::from_raw(pointer) };
            drop(reboxed);
        }

        result
    }
}
