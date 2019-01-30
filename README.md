# unique

A Rust crate containing allocators which create exactly one unique, shared pointer per distinct object.
Useful for applications with highly-redundant or deeply nested data structures such as compilers, or automatic theorem provers.
