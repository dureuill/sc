Schroedinger, An experimental library for references with erased lifetime, in Rust
==================================================================================

**NOTE: Code in this repository is experimental, and known to be unsafe.**

`Sc<T>` is similar to `&T`, but can be stored in a structure without being parameterized by a lifetime.

Usage example:
```rust
use sc::Sc;

struct A {
    sc : Sc<String>,
}

let a = A { sc : Sc::new() };
assert!(a.sc.is_none());
{
    let s = String::from("foo");
    let _dropper = a.sc.set(&s); // store a reference to s
    assert!(!a.sc.is_none());
    a.sc.map(|x| println!("{}", x)); // use the reference, prints "foo"
}
assert!(a.sc.is_none());
```

See [the example](examples/observer) for a more complex use.

`Sc` works without any allocation. The trade-off is lesser ergonomics (need to call `visit` with a closure) and a dynamic check of the validity of the reference.

See also this [message](explanation.md) explaining the idea behind `Sc`. 
