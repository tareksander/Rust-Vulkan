# Borrow Rules

The borrowing rules for RSL are a lot simpler and more restrictive than Rust's (for simplicity of Compiler development). However, like Rust, the precise model could change in the future.

For now:
- There can be either a mutable or an arbitrary number of immutable borrows.
- While borrowed, a value cannot be written to through the owning definition.
- While mutable borrowed, a value cannot be read through the owning definition.
- A borrow lasts for at least the statement it was created in (to allow borrowing of temporary values) and to the end of the statement of its last use.


Additionally:
- References to uniform buffer values have static lifetime and are always immutable.
- References to storage buffer values have static lifetime and can be mutable.
    - The respective value can only be changed through the reference for the lifetime of the borrow.



Example:

````Rust
let mut x: u32 = 0;
let a = &mut x;
let b = &x; // Error: x is already borrowed by a
*a = 2;
````


