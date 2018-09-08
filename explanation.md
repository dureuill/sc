**NOTE: This explanation is a bit outdated, please look at [the code](src/lib.rs) for the up-to-date version.**

Type erasure is a known technique of object-oriented languages, where the knowledge of the actual type of an object is "erased" and replaced by a more generic one.
This allows to trade a runtime check of the actual type (through a vtable, or a typeid) for a more homogeneous type, that can e.g. be stored in collections with other typed-erased objects.

In Rust, type erasure is expressed under the form of [`dyn Trait`](https://doc.rust-lang.org/book/second-edition/ch17-02-trait-objects.html#using-trait-objects-that-allow-for-values-of-different-types).

This repository is inspired by a thought experiment: *What is the lifetime equivalent to type erasure?*.
The easy answer would be [`Rc<T>`](https://doc.rust-lang.org/std/rc/index.html). After all, using `Rc`, you share ownership of a value, extending its lifetime dynamically through its reference count, as much as is needed.

However, `Rc` has some runtime drawbacks: in particular, both its reference count and its inner type must be allocated. Also, `Rc` expresses shared *ownership*. But what about dynamic lifetime for *non-owning references*?

Enters Schroedinger's cat. In the famous thought experiment, a cat is in a box, with a poisonous gas that can be released to kill the cat. There's a 50/50 probability that the gas be released, in which case, the cat will be dead. But the other 50% of the time, the cat will be alive. The only way to know is to open the box.

I propose exactly the same mechanism for dynamically erased lifetimes: a new type `Sc<T>` (for *Schroedinger*), that may contain either some valid reference, or none, but doesn't expose its lifetime. The only way to know if the reference is still valid is to *open the box* and query it dynamically.

A naive attempt
===============

So, according to this idea, the dream interface for `Sc<T>` is something like the following:
```rust
struct Sc<T> { /* fields omitted */ }

impl<T> Sc<T> {
    fn new(val: &T) -> Sc<T>;
    fn get(&self) -> Option<&T>;
}
```
In the above, our `Sc` is populated with a reference upon creation with `Sc::new()`, and then the reference may be recovered using `Sc::get()`, which returns an option: either the reference that was passed to `new()` if it is still alive, or `None`.
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

Of course, this doesn't work. Storing directly the reference in `Sc<T>` forces us to declare a lifetime for the struct, and thus its lifetime won't be erased anymore! 

A raw pointer attempt
=====================

Since directly storing a reference doesn't work, what about using raw pointers? Raw pointers have the advantage that they don't require any of these pesky lifetimes: we can store raw pointers all day without ever declaring a lifetime.

Let's modify our impl to use raw pointers:

```rust
use std::mem;

struct Sc<T> {
    val : *const T
}

impl<T> Sc<T> {
    fn new(val: &T) -> Sc<T> {
        Sc { val : val as *const T }
    }

    fn get<'a>(&'a self) -> Option<&'a T> {
        unsafe {
            Some(mem::transmute(self.val))
        }
    }
}
```

OK, what's the deal with `unsafe` and [`mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html)? Well, by casting the reference that is passed to `Sc::new()` to a pointer, we are effectively removing its lifetime (this is our goal, after all). So, to cast this raw pointer back to a reference with some lifetime (here, `'a`), we must use `mem::transmute`, that tells the compiler "trust the programmer, you can convert to this type!". Since this allows to cast to any possible type, the compiler cannot check that the cast is memory safe, hence `mem::transmute` must appear in an `unsafe` block.

Well, this will compile. But we used unsafe, so we should wonder how unsafe this is. The answer is **very**. What we did is incredibly hazardous. Indeed, using `Sc::get()`, the user can produce any lifetime they want, as long as it doesn't outlive the `Sc` instance itself. For instance, this trivially wrong code compiles fine:

```rust
let sc;
{
    let s = String::from("foo");
    sc = Sc::new(&s);
    println!("{}", sc.get().unwrap()); // this will print "foo"
}
println!("{}", sc.get().unwrap()); // uh, oh
```

By the time we call `sc.get()` to print the string, it has already been dropped!
This is mightlily UB.
Also, note that if things were so simple we wouldn't bother returning an `Option` from `Sc::get()`, since the reference would always be valid.
We need to constrain the reference in some other way.

Introducing Dropper type
========================

The problem with our last example is that we got a little heavy-handed with the
eraser, and it became impossible to know whether our reference was still valid
or not.

To solve this problem, we need to keep the original lifetime *somewhere*. And we
will need a way to signal to `Sc<T>` that its reference is dead as soon as it
expires.

To do so, let's introduce a second struct: `Dropper`. Basically, the `Dropper`
instance is responsible for keeping the lifetime `'object` of our original reference, 
and for signaling to our `Sc` when the reference expires.

Let's try to implement this.

```rust
use std::marker::PhantomData;

struct Dropper<'object, 'sc, T : 'object + 'sc> {
    sc : &'sc mut Sc<T>,
    _phantom : PhantomData<&'object T>,
}
```

Let's explain this definition: sc contains a mutable reference to a
`Sc<T>` with lifetime `'sc`, that we will use to tell our Sc that its reference
expired.
But what is this ghastly sight: `PhantomData`? It is a struct that we use to
record the lifetime information. Lifetime-wise, the resulting struct behaves
"as-if" it would contain a reference of type `&'object T`, except that... well,
it doesn't. `PhantomData` has a size of 0, so its inclusion doesn't make our
`Dropper` any fatter.

Now that we've got a `Dropper` that retains when its associated reference
will expire, when should we actually notify our `Sc`?
If we want to maximize the lifetime of our reference, the last possible
moment where we can notify `Sc` is when `Dropper` is dropped.
So, let's implement `Drop` for `Dropper`.

```rust
use std::ptr;
impl<'object, 'sc, T> Drop for Dropper<'object, 'sc, T> {
    fn drop(&mut self) {
        self.sc.val = ptr::null();
    }
}

impl<T> Sc<T> {
    fn get<'a>(&'a self) -> Option<&'a T> {
        unsafe {
            if ptr::eq(ptr::null(), self.val) {
                None
            } else {
                Some(mem::transmute(self.val))
            }
        }
    }
}
```

When dropping our `Dropper` object, we mutate the pointer of its associated `Sc` instance so that it points to null. We then modify `Sc::get` implementation so that a null pointer results in `None`.

This looks good, but one issue remains. How do we tie our `Sc` instance to a
`Dropper`?

First concession to our ideal design: the set() method
======================================================

Our naive `Sc::new()` function still creates a pointer from a reference by
erasing its lifetime, so we need to modify it to introduce a dropper that will retain the lifetime of this reference.
Unfortunately, the following won't work:

```rust
impl<T> Sc<T> {
    fn new<'object, 'sc>(val : &'object T) -> (Sc<T>, Dropper<'object, 'sc, T>) {
        let sc = Sc { val : val as *const T};
        (sc, Dropper { sc : &mut sc, _phantom : PhantomData })
    }
}
```

The problem with this design is that we are borrowing `&sc` from inside the
function. This borrow cannot last for longer than the scope of the function. But
our dropper needs to live for the hopefully longer lifetime `'sc`.

I don't really know how I could solve this problem while keeping such a `Sc::new()` function. So, I resorted to modifying how we construct a `Sc`: first, construct an empty Sc with `Sc::new()`, then, "fill it" with a reference by calling a `Sc::set()` method. Here's the result:

```rust
impl<T> for Sc<T> {
    fn new() -> Self {
        Sc { val : ptr::null() }
    }

    pub fn set<'sc, 'object>(&'sc mut self, val : &'object T) -> Dropper<'object, 'sc, T> { 
        self.val = val as *const T;
        Dropper { sc: self, _phantom: PhantomData }
    }
}
```

This compiles! Also, since we get our dropper returned from the set method, we
are now sure that our pointer will be set to null as soon as the dropper gets
out of scope (more on that later, though).
All's well that ends well. That's all, folks.

Or so you think.

This design may look correct, but things start to fall apart as soon as you try to actually use
it. To get a sense of why this is, let's write an example:

```rust
let mut sc = Sc::new();
assert_eq!(sc.get(), None);
{
    let s = String::from("foo");
    let _dropper = sc.set(&s);
    assert_eq!(sc.get(), Some(&s));
}
assert_eq!(sc.get(), None);
```

In this example, we declare a `Sc` in the outer scope. Initially, it is
empty, but then, in an inner scope, we set it to contain a freshly created
`String`. We can check that it indeed contains that `String`.

Lastly, after we exit the inner scope, we can check that our `Sc` is empty
again.

However, this example doesn't compile in the current design. The reason why is
that we are borrowing `sc` mutably when calling `sc.set`, and this mutable borrow
lasts until `_dropper` is dropped. While borrowing mutably, we cannot call
`sc.get()`, as this would result in a second, immutable, borrow, which is not possible when
we already have a mutable one. Shoot.

Second concession to our ideal design: interior mutability
==========================================================

There's still a way out, however, but it will cost us some API purity.
If we can't have a second borrow because the first one is mutable, then let's
make the first one immutable. This requires changing the signature of `Sc::set(=`
to take `&self` rather than `&mut self`.
To do so, we need to introduce interior mutability to our `Sc` struct.

```rust
use std::marker::PhantomData;
use std::mem;
use std::cell::Cell;
use std::ptr;

pub struct Sc<T> {
    val: Cell<*const T>,
}

#[must_use]
pub struct Dropper<'object, 'sc, T: 'object + 'sc> {
    sc: &'sc Sc<T>,
    _phantom: PhantomData<&'object T>
}

impl<'object, 'sc, T> Drop for Dropper<'object, 'sc, T> {
    fn drop(&mut self) {
        self.sc.val.set(ptr::null());
    }
}

impl<T> Sc<T> {
    pub fn new() -> Self {
        Sc { val: Cell::new(ptr::null()) }
    }

    pub fn set<'sc, 'object>(&'sc self, val: &'object T) -> Dropper<'object, 'sc, T> {
        self.val.set(val as *const T);
        Dropper { sc: self, _phantom: PhantomData }
    }

    pub fn get<'a>(&'a self) -> Option<&'a T> {
        unsafe {
            if ptr::eq(ptr::null(), self.val.get()) {
                None
            } else {
                Some(mem::transmute(self.val.get()))
            }
        }
    }
}
```

This is our final design. We simply replaced our pointer by a Cell to a pointer
in `Sc`'s definition, then we replaced all assignments to the pointer by calls to `Cell::set`, and all reads of the pointer by calls to `Cell::get`.

This time, our code compiles and behaves as expected, but at a significant API
cost: our `Sc::set()` method pretends not to modify the `Sc`, which feels "wrong",
semantically speaking.

Wrapping up
===========

Let's recap where we arrived to:

* A reentrant `Sc<T>` type that is able to dynamically know whether or not it
  contains a valid reference without containing lifetime, with 0 allocation.
* We do this by "relocating" our lifetime to a different struct that doesn't
  contain the reference, but contains a reference to our `Sc<T>`. This struct is
  responsible for telling the `Sc` when its reference is not valid anymore. This
  has significant consequences: 
  * The `Sc::set()` method **must** accept `Sc` by immutable reference,
    otherwise we won't be able to call `Sc::get()` while the inner reference
    is alive.
  * The `Sc` instance won't be movable as soon as there is an active
   `Dropper` instance tied to it. This is because moving the `Sc` instance
    would result in the `Dropper` having a dangling reference to its `Sc`
    instance (with disastrous consequences)
  * Perhaps more annoyingly, the current API is not safe with regards to the
    [leakpocalypse](http://cglab.ca/~abeinges/blah/everyone-poops/). A call to [`mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html) with an instance of `Dropper`
    would condemn the corresponding `Sc` to maybe outlive its referee. I
    believe this problem is inherent to dynamically erased lifetimes, but
    I'd love to be proven wrong!
