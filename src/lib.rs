//! Allocators which create one unique, shared pointer per distinct object.
//! Useful for applications with highly-redundant data structures such as compilers or automatic theorem provers.
//!
//! If `t1 == t2` (as determined by the allocator), then `Id::new(t1)` is pointer-equal to `Id::new(t2)`.
//! This property reduces memory use, reduces allocator hits, and allows for short-circuiting operations such as `Eq` and `Hash` by using the pointer rather than the data.
//!
//! Occasionally you may wish to "garbage collect" unused objects.
//! This can be achieved with `Allocator::delete_unused`.
//!
//! # Example
//! ```rust
//! use unique::{Allocated, Id, make_allocator};
//! use unique::allocators::HashAllocator;
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum Expr {
//!     Const(i32),
//!     Add(Id<Expr>, Id<Expr>),
//! }
//! make_allocator!(Expr, EXPR_ALLOC, HashAllocator);
//!
//! #[test]
//! fn example() {
//!     use Expr::*;
//!
//!     // Equivalent ways of allocating a `2` object.
//!     let two_x = Expr::allocator().allocate(Const(2));
//!     let two_y = EXPR_ALLOC.allocate(Const(2));
//!     let two_z = Id::new(Const(2));
//!     assert_eq!(*two_x, *two_y, *two_z, Const(2));
//!     assert_eq!(two_x, two_y, two_z);
//!
//!     // A distinct object, 2 + 2.
//!     let four = Id::new(Add(two_x.clone(), two_y.clone()));
//!     assert_ne!(two_x, four);
//!
//!     // Note only two allocations.
//!     assert_eq!(EXPR_ALLOC.allocations(), 2);
//!
//!     std::mem::drop(four);
//!
//!     // Still two allocations.
//!     assert_eq!(EXPR_ALLOC.allocations(), 2);
//!     EXPR_ALLOC.delete_unused();
//!     // Now four is no more.
//!     assert_eq!(EXPR_ALLOC.allocations(), 1);
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
#[derive(Default, PartialOrd, Ord)]
pub struct Id<T>(Arc<T>);

impl<T> Id<T> {
    /// Produce a unique integral identifier from an `Id`
    pub fn id(p: &Self) -> usize {
        &*p.0 as *const T as usize
    }

    /// Consumes the `Id` and produces a raw pointer.
    /// Must be converted back with `from_raw` to avoid a leak.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn into_raw(p: Self) -> *const T {
        Arc::into_raw(p.0)
    }

    /// Must have previously been produced by `Id::into_raw`.
    pub unsafe fn from_id(id: usize) -> Id<T> {
        let ptr = id as *const T;
        let arc = Arc::from_raw(ptr);
        Id(arc)
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

impl<T: fmt::Debug> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
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

/// `make_allocator!(Type, NAME, Allocator)`
///
/// Performs the following steps:
/// - Create a static reference to an `Allocator<Type>` accessible by `NAME`.
/// - Lazily initialise (via `lazy_static`) to `Allocator::default()`.
/// - Implements `Allocated` for `Type` by using this allocator.
#[macro_export]
macro_rules! make_allocator {
    ($type:ty, $name:ident, $alloc:ident) => {
        lazy_static::lazy_static! {
            static ref $name: $alloc<$type> = $alloc::default();
        }

        impl $crate::Allocated for $type {
            type Alloc = $alloc<$type>;
            fn allocator() -> &'static Self::Alloc {
                &$name
            }
        }
    };
}
