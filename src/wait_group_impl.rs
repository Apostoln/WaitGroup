use std::fmt;
use std::sync::{Condvar, Mutex};

pub struct WaitGroupImpl {
    counter: Mutex<usize>,
    condition: Condvar,
}

impl WaitGroupImpl {
    pub fn new() -> WaitGroupImpl {
        WaitGroupImpl {
            counter: Mutex::new(0),
            condition: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut count = self.counter.lock().unwrap();
        while *count > 0 {
            count = self.condition.wait(count).unwrap();
        }
    }

    pub fn increment_counter(&self) {
        let mut count = self.counter.lock().unwrap();
        *count += 1;
        self.notify_if_empty(*count);
    }

    pub fn add_count(&self, delta: usize) {
        let mut count = self.counter.lock().unwrap();
        *count += delta;
        self.notify_if_empty(*count);
    }

    pub fn done(&self) {
        let mut count = self.counter.lock().unwrap();
        if *count > 0 {
            *count -= 1;
            self.notify_if_empty(*count);
        }
    }

    pub fn notify_if_empty(&self, count: usize) {
        if count == 0 {
            self.condition.notify_all();
        }
    }
}

impl fmt::Debug for WaitGroupImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let count = self.counter.lock().unwrap();
        write!(f, "WaitGroup {{ count {:?} }}", *count)
    }
}