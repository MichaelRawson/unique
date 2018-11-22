use crate::backing::HashBacking;
use crate::*;

use std::fmt;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Expr {
    Const(i32),
    Add(Uniq<Expr>, Uniq<Expr>),
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
    fn unique(value: Self) -> Uniq<Self> {
        EXPR_BACKING.unique(value)
    }
}

#[test]
fn example() {
    assert!(EXPR_BACKING.num_entries() == 0);
    let two_x = uniq!(Const(2));
    let two_y = uniq!(Const(2));
    let three = uniq!(Const(3));
    assert!(EXPR_BACKING.num_entries() == 2);

    assert!(two_x == two_y);
    assert!(two_x.as_ref() as *const Expr == two_y.as_ref() as *const Expr);
    assert!(two_x != three);

    let four = uniq!(Add(two_x, two_y));
    assert!(*four == Add(uniq!(Const(2)), uniq!(Const(2))));
    assert!(format!("{}", four) == "(2 + 2)");

    assert!(EXPR_BACKING.num_entries() == 3);

    let two_z = Uniq::reuse(two_x, Const(2));
    let three = Uniq::reuse(two_x, Const(3));
    assert!(two_x.as_ref() as *const Expr == two_z.as_ref() as *const Expr);
    assert!(two_x != three);
}
