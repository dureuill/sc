extern crate sc;

use sc::Dropper;
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
            match observer.get() {
                Some(observer) => {
                    println!("Notifying {}", &observer.name);
                    observer.notify(log)
                }
                None => {}
            }
        }
    }

    pub fn register_observer<'observer, 'sc>(
        &'sc self,
        observer: &'observer Observer,
    ) -> Option<Dropper<'observer, 'sc, Observer>> {
        let sc = self
            .observers
            .iter()
            .find(|observer| observer.get().is_none())?;
        Some(sc.set(observer))
    }
}

fn main() {
    let visitable = Visitable::new(10);
    visitable.add_log("Lost for science!");
    {
        let observer = Observer::new(String::from("Toto"));
        let _dropper = visitable.register_observer(&observer).unwrap();
        visitable.add_log("Registered log");
        {
            let observer = Observer::new(String::from("Titi"));
            let _dropper = visitable.register_observer(&observer).unwrap();
            visitable.add_log("a second one");
            observer.print();
        }
        visitable.add_log("and another one!");
        observer.print();
    }
    visitable.add_log("Lost for science!");
}
