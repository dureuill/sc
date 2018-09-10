use std::cell::Cell;
use std::marker::PhantomData;
use std::mem;

pub struct Sc<T : ?Sized>(Cell<Option<*const T>>);

#[must_use]
pub struct Dropper<'object, 'sc, T: ?Sized + 'object + 'sc> {
    sc: &'sc Sc<T>,
    _phantom: PhantomData<&'object T>,
}

impl<'object, 'sc, T : ?Sized> Drop for Dropper<'object, 'sc, T> {
    fn drop(&mut self) {
        self.sc.0.set(None);
    }
}

impl<T : ?Sized> Sc<T> {
    pub fn new() -> Self {
        Sc { 0: Cell::new(None) }
    }

    pub fn set<'sc, 'object>(&'sc self, val: &'object T) -> Dropper<'object, 'sc, T> {
        self.0.set(Some(val as *const T));
        Dropper {
            sc: self,
            _phantom: PhantomData,
        }
    }

    unsafe fn get<'a>(&'a self) -> Option<&'a T> {
        self.0.get().map(|val| mem::transmute(val))
    }

    pub fn is_none(&self) -> bool {
        self.0.get().is_none()
    }

    pub fn map<'a, U, F: Fn(&'a T) -> U>(&'a self, f: F) -> Option<U> {
        unsafe { self.get().map(|x| f(x)) }
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
