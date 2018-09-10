extern crate sc;

use sc::Dropper;
use sc::Sc;
use std::cell::RefCell;

pub trait Observer {
    fn notify(&self, log: &str);
    fn name(&self) -> &str;
}

pub struct StringObserver {
    name: String,
    logs: RefCell<String>,
}

impl Observer for StringObserver {
    fn notify(&self, log: &str) {
        let mut value = self.logs.borrow_mut();
        *value += log;
        *value += "\n ";
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl StringObserver {
    pub fn print(&self) {
        println!(
            "{} received the following logs:\n {} \n",
            &self.name,
            self.logs.borrow()
        );
    }

    pub fn new(name: String) -> Self {
        Self {
            name,
            logs: RefCell::new(String::new()),
        }
    }
}

pub struct EchoObserver {
    name: String,
}

impl EchoObserver {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }
}

impl Observer for EchoObserver {
    fn notify(&self, log: &str) {
        println!("{}: Direct notification: {}", &self.name, log);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

struct Visitable {
    observers: Vec<Sc<Observer + 'static>>,
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
                println!("Notifying {}", &observer.name());
                observer.notify(log)
            });
        }
    }

    pub fn register_observer<'observer, 'sc>(
        &'sc self,
        observer: &'observer (Observer + 'static),
    ) -> Option<Dropper<'observer, 'sc, Observer + 'static>> {
        let sc = self.observers.iter().find(|observer| observer.is_none())?;
        Some(sc.set(observer))
    }
}

fn main() {
    let visitable = Visitable::new(10);
    visitable.add_log("Lost for science!");
    {
        let direct_observer = EchoObserver::new(String::from("Foo"));
        let _direct_dropper = visitable.register_observer(&direct_observer);
        let observer = StringObserver::new(String::from("Bar"));
        let _dropper = visitable.register_observer(&observer).unwrap();
        visitable.add_log("Registered log");
        {
            let observer = StringObserver::new(String::from("Baz"));
            let _dropper = visitable.register_observer(&observer).unwrap();
            visitable.add_log("a second one");
            observer.print();
        }
        visitable.add_log("and another one!");
        observer.print();
    }
    visitable.add_log("Lost for science!");
}
