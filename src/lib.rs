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
//! #[macro_use] extern crate lazy_static;
//! #[macro_use] extern crate unique;
//!
//! use unique::{Backed, Uniq};
//! use unique::backing::HashBacking;
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum Expr {
//!     Const(i32),
//!     Add(Uniq<Expr>, Uniq<Expr>),
//! }
//!
//! lazy_static! {
//!     static ref EXPR_BACKING: HashBacking<Expr> = HashBacking::new(100);
//! }
//!
//! impl Backed for Expr {
//!     fn unique(value: Self) -> Uniq<Self> {
//!         EXPR_BACKING.unique(value)
//!     }
//! }
//!
//! fn example() {
//!     let two_x = uniq!(Expr::Const(2));
//!     let two_y = uniq!(Expr::Const(2));
//!     assert!(two_x.as_ref() as *const Expr == two_y.as_ref() as *const Expr);
//! }
//! ```

extern crate chashmap;
#[macro_use]
extern crate lazy_static;
lazy_static! {}

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Data structures for implementing backing stores.
pub mod backing;

#[cfg(test)]
mod tests;

/// A type which has some backing store.
///
/// Allows the use of `Uniq::new`
pub trait Backed {
    fn unique(value: Self) -> Uniq<Self>;
}

/// A unique pointer to data, allocated by a backing store.
///
/// By "unique" I mean that if `t1 == t2` (as determined by the backing store), then
/// `Uniq::new(t1)` is pointer-equal to `Uniq::new(t2)`.
/// This property reduces memory use, reduces allocator hits, and allows for short-circuiting many operations, including `Ord` (pointer ordering), `Eq` (pointer equality), and `Hash` (pointer hash).
pub struct Uniq<T: ?Sized>(*const T);

unsafe impl<T> Send for Uniq<T> {}
unsafe impl<T> Sync for Uniq<T> {}

impl<T: Backed> Uniq<T> {
    /// Ask the backing store for a uniq'd pointer to `data`.
    ///
    /// This may be newly-allocated or recycled.
    pub fn new(data: T) -> Uniq<T> {
        T::unique(data)
    }
}

impl<T: Backed> From<T> for Uniq<T> {
    fn from(t: T) -> Self {
        Uniq::new(t)
    }
}

impl<T> Clone for Uniq<T> {
    fn clone(&self) -> Self {
        Uniq(self.0)
    }
}

impl<T> Copy for Uniq<T> {}

impl<T> PartialEq for Uniq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Uniq<T> {}

impl<T> PartialOrd for Uniq<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Uniq<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Uniq<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}

impl<T> Deref for Uniq<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> AsRef<T> for Uniq<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for Uniq<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: fmt::Debug> fmt::Debug for Uniq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Uniq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T> fmt::Pointer for Uniq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.0)
    }
}

/// Shorthand for `Uniq::new`.
#[macro_export]
macro_rules! uniq {
    ($e:expr) => {Uniq::new($e)}
}
