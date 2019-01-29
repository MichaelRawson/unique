//! Allows the creation of pointers which are guaranteed to be equal pointers for equal data.
//!
//! Useful for applications with highly-redundant or deeply nested data structures such as
//! compilers or automatic theorem provers.
//!
//! In future this crate may support recovery of allocated memory that is no longer required (via e.g. mark-and-sweep), but
//! for now all backing stores leak memory as required.
//!
//! # Example
//! ```rust
//! use lazy_static::lazy_static;
//! use unique::{Backed, Id};
//! use unique::backing::HashBacking;
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum Expr {
//!     Const(i32),
//!     Add(Id<Expr>, Id<Expr>),
//! }
//!
//! lazy_static! {
//!     static ref EXPR_BACKING: HashBacking<Expr> = HashBacking::new(100);
//! }
//!
//! impl Backed for Expr {
//!     fn unique(value: Self) -> Id<Self> {
//!         EXPR_BACKING.unique(value)
//!     }
//! }
//!
//! #[test]
//! fn example() {
//!     let two_x = Id::new(Expr::Const(2));
//!     let two_y = Id::new(Expr::Const(2));
//!     assert!(two_x.as_ref() as *const Expr == two_y.as_ref() as *const Expr);
//! }
//! ```

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr::NonNull;

/// Data structures for implementing backing stores.
pub mod backing;

#[cfg(test)]
mod tests;

/// A type which has some backing store.
///
/// Allows the use of `Id::new`
pub trait Backed {
    fn unique(value: Self) -> Id<Self>;
}

/// A unique pointer to data, allocated by a backing store.
///
/// By "unique" I mean that if `t1 == t2` (as determined by the backing store), then
/// `Id::new(t1)` is pointer-equal to `Id::new(t2)`.
/// This property reduces memory use, reduces allocator hits, and allows for short-circuiting operations such as `Eq` (pointer equality instead of object equality), and `Hash` (pointer hash instead of object hash).
pub struct Id<T: ?Sized>(NonNull<T>);

unsafe impl<T> Send for Id<T> {}
unsafe impl<T> Sync for Id<T> {}

impl<T> Id<T> {
    /// Produce an integral ID from an `Id`.
    pub fn id(p: Self) -> usize {
        p.0.as_ptr() as usize
    }
}

impl<T: Backed> Id<T> {
    /// Ask the backing store for a uniq'd pointer to `data`.
    ///
    /// This may be newly-allocated or recycled.
    pub fn new(data: T) -> Id<T> {
        T::unique(data)
    }
}

impl<T: Backed + Eq> Id<T> {
    /// Attempt to re-use this pointer for `data` if is value-equal, or allocate if not.
    ///
    /// Useful over `Id::new` as a performance optimisation.
    pub fn reuse(p: Self, data: T) -> Self {
        if *p == data {
            p
        } else {
            Id::new(data)
        }
    }
}

impl<T: Backed> From<T> for Id<T> {
    fn from(t: T) -> Self {
        Id::new(t)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id(self.0)
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T: PartialOrd> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: Ord> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}

impl<T> Deref for Id<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> AsRef<T> for Id<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for Id<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: fmt::Debug> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T> fmt::Pointer for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.0)
    }
}
