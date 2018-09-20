//!
//! The `sc` module provides the `Sc` type, an "Almost-Safe" reference type with a dynamic lifetime.
//!
//! A `Sc<T>` is similar to a `&'a T`, but can be stored in a struct without tying the `'a` lifetime to the struct.
//! In exchange for that, a dynamic check is performed to ensure that the reference is still alive when it is accessed.
//!
//! # Examples
//!
//! A `Sc<T>` is always empty upon initialization.
//! ```rust
//! use sc::Sc;
//!
//! let sc = Sc::<String>::new();
//! assert!(sc.is_none()); // `Sc` is initially empty.
//! ```
//!
//! To set a `Sc` to reference something, one must create a `Locker` object.
//! The `Locker` object is responsible for keeping the lifetime of the referenced object.
//! To tie a `Sc` to a reference, one must call the `Locker::lock` method on a `Locker` object.
//! ```rust
//! use sc::Sc;
//! use sc::Locker;
//!
//! let sc = Sc::new();
//! assert!(sc.is_none()); // `Sc` is initially empty.
//! {
//!     let s = String::from("foo");
//!     let mut locker = Locker::new(); // create a `Locker` object.
//!     unsafe { locker.lock(&s, &sc) }; // This call sets the sc to reference `s`
//!     assert!(sc.is_some()); // `Sc` now contains something!
//! }
//! ```
//!
//! The `Sc` remains tied to the reference for the **entire lifetime** of the `Locker` instance,
//! then it becomes empty again.
//!
//! ```rust
//! use sc::Sc;
//! use sc::Locker;
//!
//! let sc = Sc::new();
//! assert!(sc.is_none()); // `Sc` is initially empty.
//! {
//!     let s = String::from("foo");
//!     let mut locker = Locker::new(); // create a `Locker` object.
//!     unsafe { locker.lock(&s, &sc) }; // This call sets the sc to reference `s`
//!     assert!(sc.is_some()); // `Sc` now contains something!
//! }
//! assert!(sc.is_none()); // empty again!
//! ```
//!
//! While the `Sc` is set to a reference, the reference can be accessed in a closure through the `map` method.
//! ```rust
//! use sc::Sc;
//! use sc::Locker;
//!
//! let sc = Sc::new();
//! assert!(sc.is_none()); // `Sc` is initially empty.
//! {
//!     let s = String::from("foo");
//!     let mut locker = Locker::new(); // create a `Locker` object.
//!     unsafe { locker.lock(&s, &sc) }; // This call sets the sc to reference `s`
//!     assert!(sc.is_some()); // `Sc` now contains something!
//!     sc.map(|x| println!("{}", x)); // will print "foo"
//! }
//! sc.map(|x| println!("{}", x)); // does nothing
//! ```
//!
//! Similarly to `Option::map`, if the closure returns a value it will be wrapped in an option that will be `Some` if
//! the `Sc` is set to a reference, and `None` otherwise.
//!
//! ```rust
//! use sc::Sc;
//! use sc::Locker;
//!
//! let sc = Sc::new();
//! assert!(sc.is_none()); // `Sc` is initially empty.
//! {
//!     let s = String::from("foo");
//!     let mut locker = Locker::new(); // create a `Locker` object.
//!     unsafe { locker.lock(&s, &sc) }; // This call sets the sc to reference `s`
//!     assert!(sc.is_some()); // `Sc` now contains something!
//!     assert_eq!(sc.map(|x| x.clone() + "bar"), Some(String::from("foobar")));
//! }
//! assert_eq!(sc.map(|x| x.clone() + "bar"), None);
//! ```
//!
//! Here is a simple use of a `Sc` to store a reference to a `String` in a struct `A` that doesn't have a lifetime:
//!  ```rust
//! use sc::Sc;
//! use sc::Locker;
//!
//! struct A { // No declared lifetime!
//!    sc : Sc<String>,
//! }
//!
//! let a = A { sc : Sc::new() }; // `Sc` is initially empty.
//! assert!(a.sc.is_none());
//! {
//!     let s = String::from("foo");
//!     let mut locker = Locker::new();
//!     unsafe { locker.lock(&s, &a.sc) }; // set `Sc` to reference the String `s`.
//!     assert!(a.sc.is_some());
//!     a.sc.map(|x| println!("{}", x)); // use the reference: prints "foo"
//! }
//! assert!(a.sc.is_none());
//! ```
//!
//! # Safety
//!
//! `Sc` relies on the `drop` method being called on the `Locker` instance to be safe. However,
//! leaking a value is considered safe is rust. As a result, the `Locker::lock` method is marked unsafe.
//! Note that it is statically impossible to call `Locker::lock` on a `Locker` instance in a `Rc` cycle, which
//! notably decreases the risk of accidental leaks.
//! See the `Locker::lock` method documentation for further details.
//!

use std::cell::Cell;

/// An "Almost-Safe" reference type with a dynamic lifetime.
///
/// `Sc` stands for `Schroedinger` because one must check to know if the reference is dead or alive.
/// See the [crate-level documentation](./index.html) for more details.
pub struct Sc<T: ?Sized>(Cell<Option<*const T>>);

struct Dropper<'sc, T: ?Sized + 'sc> {
    sc: &'sc Sc<T>,
}

impl<'sc, T: ?Sized> Drop for Dropper<'sc, T> {
    fn drop(&mut self) {
        self.sc.0.set(None)
    }
}

struct Marker;

/// The object responsible for the lifetime of the reference set in `Sc`.
///
/// See the [crate-level documentation](./index.html) for more details.
pub struct Locker<'sc, 'auto, T: ?Sized + 'sc + 'auto> {
    sc: Option<Dropper<'sc, T>>,
    marker: Marker,
    autoref: Option<&'auto Marker>,
}

impl<'sc, 'auto, T: ?Sized> Locker<'sc, 'auto, T> {
    /// Constructs a new `Locker<T>`.
    pub fn new() -> Self {
        Self {
            sc: None,
            marker: Marker,
            autoref: None,
        }
    }

    /// Lock the passed reference `val` with the passed `sc` for the **entire lifetime** of this `Locker`.
    ///
    /// # Safety
    ///
    /// This method is unsafe, because the passed `sc` relies on `drop` being called for the `Locker` instance,
    /// and leaking an object without calling `drop` is considered safe in Rust.
    ///
    /// To prevent any possible access to a dangling reference with the `Sc::map` method,
    /// this `Locker` instance must verify the following preconditions before calling `Locker::lock`:
    ///   * It must not be wrapped in a `ManuallyDrop` wrapper.
    ///   * It must not be put in a Box that is leaked.
    ///   * It must not be put in a cycle (like a `Rc` cycle). Fortunately, this is impossible to do using safe Rust.
    ///   * (once Drop objects in enum land in stable) It must not be a member of a `union`
    ///
    /// # Examples
    ///
    /// Because it is set for the entire lifetime of the `Locker`, it is statically impossible to `lock` a `Locker`
    /// to a reference of a shorter lifetime.
    ///
    /// Note that it is impossible call `Locker::lock()` on a locker in a cycle.
    ///
    /// Note that it is also impossible to call `mem::forget()` on a locker instance for which `Locker::lock()` has been called.
    ///
    pub unsafe fn lock(&'auto mut self, val: &'auto T, sc: &'sc Sc<T>) {
        let ptr = val as *const T;
        sc.0.set(Some(ptr));
        self.sc = Some(Dropper { sc });
        self.autoref = Some(&self.marker);
    }
}

impl<T: ?Sized> Sc<T> {
    pub fn new() -> Self {
        Self { 0: Cell::new(None) }
    }

    unsafe fn get(&self) -> Option<&T> {
        self.0.get().map(|ptr| &*ptr)
    }

    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Option<U> {
        unsafe { self.get().map(f) }
    }

    pub fn is_some(&self) -> bool {
        self.0.get().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.get().is_none()
    }
}

pub struct Wrapper<'sc, 'auto, T: 'sc + 'auto> {
    locker: Locker<'sc, 'auto, T>,
    data: T,
}

impl<'sc, 'auto, T> Wrapper<'sc, 'auto, T> {
    pub fn new(data: T) -> Self {
        Self {
            locker: Locker::new(),
            data,
        }
    }

    pub unsafe fn lock(this: &'auto mut Self, sc: &'sc Sc<T>) {
        this.locker.lock(&this.data, sc);
    }
}

use std::ops::Deref;
use std::ops::DerefMut;
impl<'sc, 'auto, T> Deref for Wrapper<'sc, 'auto, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'sc, 'auto, T> DerefMut for Wrapper<'sc, 'auto, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sc = Sc::new();
        assert!(sc.is_none());
        {
            let s = String::from("foo");
            {
                let mut locker = Locker::new();
                unsafe {
                    locker.lock(&s, &sc);
                }
                assert!(!sc.is_none());
            }
            assert!(sc.is_none());
            {
                let mut locker = Locker::new();
                unsafe {
                    locker.lock(&s, &sc);
                }
                assert!(!sc.is_none());
            }
            assert!(sc.is_none());
        }
        assert!(sc.is_none());
    }

    #[test]
    fn with_wrapper() {
        let sc = Sc::new();
        assert!(sc.is_none());
        {
            let mut s = Wrapper::new(String::from("bar"));
            {
                unsafe {
                    Wrapper::lock(&mut s, &sc);
                }
                assert!(!sc.is_none());
            }
            // actually here we are still locked because a wrapper automatically locks for its whole lifetime
            assert!(!sc.is_none());
        }
        assert!(sc.is_none());
    }
}
