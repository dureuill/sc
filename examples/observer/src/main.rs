extern crate sc;

use sc::Locker;
use sc::Sc;
use std::cell::RefCell;

struct Observer {
    name: String,
    logs: RefCell<String>,
}

impl Observer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            logs: RefCell::new(String::new()),
        }
    }

    pub fn notify(&self, log: &str) {
        let mut value = self.logs.borrow_mut();
        *value += log;
        *value += "\n ";
    }

    pub fn print(&self) {
        println!(
            "{} received the following logs:\n {} \n",
            &self.name,
            self.logs.borrow()
        );
    }
}

struct Visitable {
    observers: Vec<Sc<Observer>>,
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
            observer.map(|observer| {
                println!("Notifying {}", &observer.name);
                observer.notify(log)
            });
        }
    }

    pub fn register_observer<'observer, 'sc>(
        &'sc self,
        observer: &'observer Observer,
        locker: &'observer mut Locker<'sc, 'observer, Observer>,
    ) -> Option<()> {
        let sc = self
            .observers
            .iter()
            .find(|observer| observer.is_none())?;
        Some(locker.lock(observer, &sc))
    }
}

fn main() {
    let visitable = Visitable::new(10);
    visitable.add_log("Lost for science!");
    {
        let observer = Observer::new(String::from("Toto"));
        let mut locker = Locker::new();
        visitable.register_observer(&observer, &mut locker);
        visitable.add_log("Registered log");
        {
            let observer = Observer::new(String::from("Titi"));
            let mut locker = Locker::new();
            visitable.register_observer(&observer, &mut locker);
            visitable.add_log("a second one");
            observer.print();
        }
        visitable.add_log("and another one!");
        observer.print();
    }
    visitable.add_log("Lost for science!");
}
