extern crate sc;

use sc::Dropper;
use sc::Sc;

struct Visitable {
    observers: Vec<Sc<Fn(&str) + 'static>>,
}

impl Visitable {
    pub fn new(observer_count: usize) -> Self {
        let mut v = Vec::new();
        v.reserve(observer_count);
        for _i in 0..observer_count {
            v.push(Sc::new());
        }
        Visitable { observers: v }
    }

    pub fn add_log(&self, log: &str) {
        for observer in &self.observers {
            observer.map(|observer| observer(log));
        }
    }

    #[must_use]
    pub fn register_observer<'observer, 'sc>(
        &'sc self,
        observer: &'observer (Fn(&str) + 'static),
    ) -> Option<Dropper<'observer, 'sc, Fn(&str) + 'static>> {
        let sc = self.observers.iter().find(|observer| observer.is_none())?;
        Some(sc.set(observer))
    }
}

fn foo_print(x: &str) {
    println!("Foo received '{}'", x)
}

fn main() {
    let visitable = Visitable::new(10);
    visitable.add_log("Lost for science!");
    {
        let _dropper = visitable.register_observer(&foo_print);
        visitable.add_log("Registered log");
        {
            let name = String::from("Bar");
            let lambda = move |log : &_| println!("{} received '{}'", name, log);
            let _dropper_2 = visitable.register_observer(&lambda);
            visitable.add_log("Registered log 2");
        }
    }
    visitable.add_log("Lost for science!");
}
