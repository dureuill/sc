Schroedinger, An experimental library for references with erased lifetime, in Rust
==================================================================================

Type erasure is a known technique in object-oriented languages, where the knowledge of the actual type of an object is "erased" and replaced by a more generic one.
This allows to trade a runtime check of the actual type (through a vtable, or a typeid) for a more homogeneous type, that can e.g. be stored in collections with other typed-erased objects.

In Rust, type erasure is expressed under the form of `dyn Trait`.

This repository starts from a thought experiment: "What is the lifetime equivalent to type erasure?".
An easy answer: `Rc<T>`. After all, using Rc, you share ownership of a value, extending its lifetime dynamically through its reference count, as much as is needed.

However, Rc has some runtime drawbacks: in particular, both its reference count and its inner type must be allocated. Also, Rc expresses shared *ownership*. But what about dynamic lifetime for *references*?

Enters Schroedinger's cat. In his famous though experiment, a cat is in box, with a bomb connected to a quantic fuse. There's a 50/50 probability for the fuse to be activated and the bomb to detonate, in which case, the cat will be dead. But the other 50% of the time, the cat will be alive. The only way to know is to open the box.

I propose exactly the same mechanism for dynamically erased lifetimes: a new type Sc<T>, that may contain either some reference, or none. The only way to know is to query it dynamically.

A naive attempt
===============

So, according to this idea, the dream interface for Sc<T> is something like the following:
```rust
struct Sc<T> { /* fields omitted */ }

impl<T> Sc<T> {
    fn new(val: &T) -> Sc<T>;
    fn get(&self) -> Option<&T>;
}
```
In the above, the Sc is populated with a reference upon creation with `new()`, and then the reference may be recovered using `get`, which returns an option: either the reference that was passed to `new()`, or `None`.
So, let's try to fill in these impls!

```rust
struct Sc<T> {
    val : &T
}

impl<T> Sc<T> {
    fn new(val: &T) -> Sc<T> {
        Sc { val }
    }

    fn get(&self) -> Option<&T> {
        Some(self.val)
    }
}
```

Of course, this doesn't work. Storing directly the reference in `Sc<T>` forces us to declare a lifetime for the struct, and thus its lifetime won't be erased anymore! Also, if this was possible, we wouldn't bother to return an `Option` from `get()`, since the reference would always be valid.

A raw pointer attempt
=====================

OK, so directly storing a reference doesn't work, but what about raw pointers? Raw pointers have the advantage that they don't require any of these pesky lifetimes. We can store a raw pointer all day without ever declaring a lifetime.

Let's modify our impl to use raw pointers:

```rust
use core::mem;

struct Sc<T> {
    val : *const T
}

impl<T> Sc<T> {
    fn new(val: &T) -> Sc<T> {
        Sc { val : val as *const T }
    }

    fn get<'a>(&'a self) -> Option<&'a T> {
        unsafe {
            Some(mem::transmute(val))
        }
    }
}
```

OK, what's the deal with `unsafe` and `mem::transmute`? Well, by casting the reference that is passed to `new()` to a pointer, we are effectively removing its lifetime (this is our goal, after all). So, to cast this raw pointer back to a reference with some lifetime (here, `'a`), we need to use `mem::transmute`, that tells the compiler "trust the programmer, you can convert to this type!". Since this allows to cast to any possible type, the compiler cannot check that the cast is memory safe, hence `mem::transmute` must appear in an `unsafe` block.

So, OK, this will compile. It is also incredibly hazardous. Indeed, using `get()`, the user can produce any lifetime as long as it doesn't outlive the `Sc` instance itself. This is trivially wrong:

```rust
let sc;
{
    let s = String::from("toto");
    sc = Sc::new(&s);
}
println!("{}", sc.get().unwrap()); // uh, oh
```
By the time we

