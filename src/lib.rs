//! Allocators which create one unique, shared pointer per distinct object.
//! Useful for applications with highly-redundant or deeply nested data structures such as compilers, or automatic theorem provers.
//!
//! If `t1 == t2` (as determined by the allocator), then `Id::new(t1)` is pointer-equal to `Id::new(t2)`.
//! This property reduces memory use, reduces allocator hits, and allows for short-circuiting operations such as `Eq` and `Hash` by using the pointer rather than the data.
//!
//! # Example
//! ```rust
//! use lazy_static::lazy_static;
//! use unique::{Allocated, Id, make_allocator};
//! use unique::allocators::HashAllocator;
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum Expr {
//!     Const(i32),
//!     Add(Id<Expr>, Id<Expr>),
//! }
//!
//! make_allocator!(Expr, __EXPR_ALLOC, HashAllocator);
//!
//! #[test]
//! fn example() {
//!     let two_x = Id::new(Expr::Const(2));
//!     let two_y = Id::new(Expr::Const(2));
//!     let three = Id::new(Expr::Const(3));
//!
//!     assert_eq!(two_x, two_y);
//!     assert_ne!(two_x, three);
//!     assert_eq!(Expr::allocator().allocations(), 2);
//! }
//! ```

use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// Possible allocators to use
pub mod allocators;

#[cfg(test)]
mod tests;

/// Allocate shared unique pointers
pub trait Allocator<T: Eq>: Default {
    /// Recycle a value if possible, or allocate a new one
    fn allocate(&self, t: T) -> Id<T>;

    /// The current number of allocations
    fn allocations(&self) -> usize;

    /// Sweep for unused values and delete them
    fn delete_unused(&self);
}

/// A type which has some allocator
///
/// Allows the use of `Id::new`
pub trait Allocated: Eq + Sized + 'static {
    type Alloc: Allocator<Self>;

    fn allocator() -> &'static Self::Alloc;
}

/// A unique, shared pointer
///
#[derive(Debug, Default, PartialOrd, Ord)]
pub struct Id<T>(Arc<T>);

impl<T> Id<T> {
    /// Produce a unique integral identifier from an `Id`
    pub fn id(p: &Self) -> usize {
        &*p.0 as *const T as usize
    }
}

impl<T: Allocated> Id<T> {
    /// Get a shared pointer to (something value-equal to) `t`
    pub fn new(t: T) -> Self {
        T::allocator().allocate(t)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id(Arc::clone(&self.0))
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Id::id(self).hash(hasher);
    }
}

impl<T> Deref for Id<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: fmt::Display> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> fmt::Pointer for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for Id<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Borrow<T> for Id<T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

#[macro_export]
macro_rules! make_allocator {
    ($type:ty, $name:ident, $alloc:ident) => {
        lazy_static! {
            static ref $name: $alloc<$type> = $alloc::default();
        }

        impl Allocated for $type {
            type Alloc = $alloc<$type>;
            fn allocator() -> &'static Self::Alloc {
                &$name
            }
        }
    };
}
