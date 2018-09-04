extern crate core;

mod with_mut {
    use std::marker::PhantomData;
    use std::mem;
    use std::cell::Cell;
    use core::ptr;

    pub struct Sc<T> {
        Dead,
        Alive(*const T),
    }

    #[must_use]
    pub struct Dropper<'object, 'sc, T: 'object + 'sc> {
        sc: &'sc mut Sc<T>,
        _phantom: PhantomData<&'object T>
    }

    impl<'object, 'sc, T> Drop for Dropper<'object, 'sc, T> {
        fn drop(&mut self) {
            *self.sc = Dead
        }
    }

    impl<T> Sc<T> {
        pub fn new() -> Self {
            Sc::Dead
        }

        pub fn set<'sc, 'object>(&'sc mut self, val: &'object T) -> Dropper<'object, 'sc, T> {
            self.val.set(val as *const T);
            Dropper { sc: self, _phantom: PhantomData }
        }

        pub fn get<'a, 'b : 'a, 'object>(&'a self,
                                         dropper : Option<Dropper<'object, 'b, T>>) -> Option<&'a T> {
            unsafe {
                match {
                    
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
            assert_eq!(sc.get(), None);
            {
                let s = String::from("foo");
                let _dropper = sc.set(&s);
                assert_eq!(sc.get(), Some(&s));
            }
            assert_eq!(sc.get(), None);
        }
    }
}

mod with_cell {
    use std::marker::PhantomData;
    use std::mem;
    use std::cell::Cell;
    use core::ptr;

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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let sc = Sc::new();
            assert_eq!(sc.get(), None);
            {
                let s = String::from("foo");
                let _dropper = sc.set(&s);
                assert_eq!(sc.get(), Some(&s));
            }
            assert_eq!(sc.get(), None);
        }
    }
}