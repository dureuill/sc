extern crate core;

use core::ptr;
use std::cell::Cell;
use std::marker::PhantomData;
use std::mem;

pub struct Sc<T> {
    val: Cell<*const T>,
}

#[must_use]
pub struct Dropper<'object, 'sc, T: 'object + 'sc> {
    sc: &'sc Sc<T>,
    _phantom: PhantomData<&'object T>,
}

impl<'object, 'sc, T> Drop for Dropper<'object, 'sc, T> {
    fn drop(&mut self) {
        self.sc.val.set(ptr::null());
    }
}

impl<T> Sc<T> {
    pub fn new() -> Self {
        Sc {
            val: Cell::new(ptr::null()),
        }
    }

    pub fn set<'sc, 'object>(&'sc self, val: &'object T) -> Dropper<'object, 'sc, T> {
        self.val.set(val as *const T);
        Dropper {
            sc: self,
            _phantom: PhantomData,
        }
    }

    unsafe fn get<'a>(&'a self) -> Option<&'a T> {
        if ptr::eq(ptr::null(), self.val.get()) {
            None
        } else {
            Some(mem::transmute(self.val.get()))
        }
    }

    pub fn is_none(&self) -> bool {
        ptr::eq(ptr::null(), self.val.get())
    }

    pub fn visit<'a, U, F: Fn(&'a T) -> U>(&'a self, f: F) -> Option<U> {
        unsafe {
            match self.get() {
                Some(x) => Some(f(x)),
                None => None,
            }
        }
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
            let _dropper = sc.set(&s);
            assert!(!sc.is_none());
        }
        assert!(sc.is_none());
    }
}
