use std::fmt;
use std::mem::drop;

use crate::allocators::HashAllocator;
use crate::{make_allocator, Allocated, Allocator, Id};

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Expr {
    Const(i32),
    Add(Id<Expr>, Id<Expr>),
}
make_allocator!(Expr, __EXPR_ALLOC, HashAllocator);

use self::Expr::*;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Const(n) => write!(f, "{}", n),
            Add(x, y) => write!(f, "({} + {})", x, y),
        }
    }
}

#[test]
fn example() {
    assert_eq!(Expr::allocator().allocations(), 0);
    let two_x = Id::new(Const(2));
    let two_y = Id::new(Const(2));
    let three = Id::new(Const(3));
    assert_eq!(Expr::allocator().allocations(), 2);

    assert_eq!(two_x, two_y);
    assert_eq!(two_x.as_ref() as *const Expr, two_y.as_ref() as *const Expr);
    assert_ne!(two_x, three);

    let four = Id::new(Add(two_x, two_y));
    assert_eq!(*four, Add(Id::new(Const(2)), Id::new(Const(2))));
    assert_eq!(format!("{}", four), "(2 + 2)");
    assert_eq!(Expr::allocator().allocations(), 3);

    drop(three);
    Expr::allocator().delete_unused();
    assert_eq!(Expr::allocator().allocations(), 2);
}
