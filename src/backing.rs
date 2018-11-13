use crate::*;

use chashmap::CHashMap;

struct HashBackingRecord<T> {
    unique: Uniq<T>,
}

/// A backing store based on a concurrent hashmap.
pub struct HashBacking<T: 'static> {
    backing: CHashMap<&'static T, HashBackingRecord<T>>,
}

impl<T> HashBacking<T> {
    /// How many items are currently stored?
    pub fn num_entries(&self) -> usize {
        self.backing.len()
    }
}

impl<T: Eq + Hash> HashBacking<T> {
    /// Create a new backing store, pre-allocating `capacity` items.
    pub fn new(capacity: usize) -> Self {
        HashBacking {
            backing: CHashMap::with_capacity(capacity),
        }
    }
}

unsafe fn force_static<T>(reference: &T) -> &'static T {
    let ptr = reference as *const T;
    &*ptr
}

impl<T: Eq + Hash> HashBacking<T> {
    /// Allows implementing `Backed` for any type that implements `Eq + Hash`.
    pub fn unique(&self, val: T) -> Uniq<T> {
        // lifetimes on CHashMap are too restrictive
        let val_ref = &val;
        let static_val = unsafe { force_static(val_ref) };

        if let Some(record) = self.backing.get(&static_val) {
            return record.unique;
        } else {
            let boxed = Box::new(val);
            let box_ref = unsafe { force_static(boxed.as_ref()) };
            let unique = Uniq(box_ref as *const T);
            let record = HashBackingRecord { unique };

            self.backing.upsert(
                box_ref,
                || {
                    let _ = Box::leak(boxed);
                    record
                },
                |_| {},
            );

            self.backing.get(&box_ref).unwrap().unique
        }
    }
}
