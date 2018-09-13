use std::cell::Cell;
use std::mem;

pub struct Sc<T>(Cell<Option<*const T>>);

struct Dropper<'sc, T: 'sc> {
    sc: &'sc Sc<T>,
}

impl<'sc, T> Drop for Dropper<'sc, T> {
    fn drop(&mut self) {
        self.sc.0.set(None)
    }
}

struct Marker;

pub struct Locker<'sc, 'auto, T: 'sc + 'auto> {
    sc: Option<Dropper<'sc, T>>,
    marker: Marker,
    autoref: Option<&'auto Marker>,
}

impl<'sc, 'auto, T> Locker<'sc, 'auto, T> {
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

impl<T> Sc<T> {
    pub fn new() -> Self {
        Self { 0: Cell::new(None) }
    }

    unsafe fn get(&self) -> Option<&T> {
        self.0.get().map(|ptr| mem::transmute(ptr))
    }

    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Option<U> {
        unsafe { self.get().map(f) }
    }

    pub fn is_none(&self) -> bool {
        unsafe { self.get().is_none() }
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
            assert!(!sc.is_none());
        }
        assert!(sc.is_none());
    }
}
