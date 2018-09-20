use std::cell::Cell;

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

pub struct Locker<'sc, 'auto, T: ?Sized + 'sc + 'auto> {
    sc: Option<Dropper<'sc, T>>,
    marker: Marker,
    autoref: Option<&'auto Marker>,
}

impl<'sc, 'auto, T: ?Sized> Locker<'sc, 'auto, T> {
    pub fn new() -> Self {
        Self {
            sc: None,
            marker: Marker,
            autoref: None,
        }
    }

    pub fn lock(&'auto mut self, val: &'auto T, sc: &'sc Sc<T>) {
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

    pub fn lock(this: &'auto mut Self, sc: &'sc Sc<T>) {
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
                locker.lock(&s, &sc);
                assert!(!sc.is_none());
            }
            assert!(sc.is_none());
            {
                let mut locker = Locker::new();
                locker.lock(&s, &sc);
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
                Wrapper::lock(&mut s, &sc);
                assert!(!sc.is_none());
            }
            // actually here we are still locked because a wrapper automatically locks for its whole lifetime
            assert!(!sc.is_none());
        }
        assert!(sc.is_none());
    }
}
