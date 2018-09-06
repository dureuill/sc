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
    observer: Sc<Observer>,
}

impl Visitable {
    pub fn new() -> Self {
        Visitable {
            observer: Sc::new(),
        }
    }

    pub fn add_log(&self, log: &str) {
        match self.observer.get() {
            Some(observer) => {
                println!("Notifying {}", &observer.name);
                observer.notify(log)
            }
            None => {
                println!("No one to notify for log: {}!", log);
            }
        };
    }

    pub fn register_observer<'observer, 'sc>(
        &'sc self,
        observer: &'observer Observer,
    ) -> Dropper<'observer, 'sc, Observer> {
        self.observer.set(&observer)
    }
}

fn main() {
    let visitable = Visitable::new();
    visitable.add_log("Lost for science!");
    {
        let observer = Observer::new(String::from("Toto"));
        let _dropper = visitable.register_observer(&observer);
        visitable.add_log("Registered log");
        visitable.add_log("and another one!");
        observer.print();
    }
    visitable.add_log("Lost for science!");
}
