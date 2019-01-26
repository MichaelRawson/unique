use lazy_static::lazy_static;

use crate::backing::HashBacking;
use crate::*;

use std::fmt;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Expr {
    Const(i32),
    Add(Id<Expr>, Id<Expr>),
}
use self::Expr::*;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Const(n) => write!(f, "{}", n),
            Add(x, y) => write!(f, "({} + {})", x, y),
        }
    }
}

lazy_static! {
    static ref EXPR_BACKING: HashBacking<Expr> = HashBacking::new(100);
}

impl Backed for Expr {
    fn unique(value: Self) -> Id<Self> {
        EXPR_BACKING.unique(value)
    }
}

#[test]
fn example() {
    assert!(EXPR_BACKING.num_entries() == 0);
    let two_x = Id::new(Const(2));
    let two_y = Id::new(Const(2));
    let three = Id::new(Const(3));
    assert!(EXPR_BACKING.num_entries() == 2);

    assert!(two_x == two_y);
    assert!(two_x.as_ref() as *const Expr == two_y.as_ref() as *const Expr);
    assert!(two_x != three);

    let four = Id::new(Add(two_x, two_y));
    assert!(*four == Add(Id::new(Const(2)), Id::new(Const(2))));
    assert!(format!("{}", four) == "(2 + 2)");

    assert!(EXPR_BACKING.num_entries() == 3);

    let two_z = Id::reuse(two_x, Const(2));
    let three = Id::reuse(two_x, Const(3));
    assert!(two_x.as_ref() as *const Expr == two_z.as_ref() as *const Expr);
    assert!(two_x != three);
}
