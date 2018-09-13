use std::cell::Cell;
use std::mem;
use std::ops::Deref;

pub struct Sc<T>(Cell<Option<*const T>>);

struct Dropper<'sc, T: 'sc> {
    sc: &'sc Sc<T>,
}

impl<'sc, T> Drop for Dropper<'sc, T> {
    fn drop(&mut self) {
        self.sc.0.set(None)
    }
}

pub struct Wrapper<'sc, 'auto, T: 'sc + 'auto> {
    data: T,
    sc: Cell<Option<Dropper<'sc, T>>>,
    autoref: Cell<Option<&'auto T>>,
}

impl<'sc, 'auto, T> Deref for Wrapper<'sc, 'auto, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'sc, 'auto, T> Wrapper<'sc, 'auto, T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            sc: Cell::new(None),
            autoref: Cell::new(None),
        }
    }

    pub fn lock(this: &'auto Self, sc: &'sc Sc<T>) {
        let ptr = &this.data as *const T;
        sc.0.set(Some(ptr));
        this.sc.set(Some(Dropper { sc }));
        this.autoref.set(Some(&this.data));
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
            let s = Wrapper::new(s);
            Wrapper::lock(&s, &sc);
            assert!(!sc.is_none());
        }
        assert!(sc.is_none());
    }
}
