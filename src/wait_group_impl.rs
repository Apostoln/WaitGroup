use std::convert::TryFrom;
use std::fmt;
use std::sync::{Condvar, Mutex};

pub enum WaitGroupError {
    NegativeCounter(isize),
    Unexpected(String),
}

impl fmt::Debug for WaitGroupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WaitGroupError::NegativeCounter(counter) => {
                write!(f, "Counter is negative during wait() call: {}", counter)
            }
            WaitGroupError::Unexpected(description) => {
                write!(f, "Unexpected WaitGroupError: {}", description)
            }
        }
    }
}

pub struct WaitGroupImpl {
    counter: Mutex<isize>,
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
        self.try_wait().unwrap(); //todo what will be printed in panic?
    }

    pub fn try_wait(&self) -> Result<(), WaitGroupError> {
        let mut count = self.counter.lock().unwrap();
        if *count < 1 {
            return Err(WaitGroupError::NegativeCounter(*count));
        }
        while *count > 0 {
            count = self.condition.wait(count).unwrap();
        }
        Ok(())
    }

    pub fn increment_counter(&self) {
        let mut count = self.counter.lock().unwrap();
        *count += 1;
        self.notify_if_empty(*count);
    }

    pub fn add_count(&self, delta: usize) {
        self.add_count_unchecked(delta as isize);
    }

    pub fn add_count_unchecked(&self, delta: isize) {
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

    pub fn notify_if_empty(&self, count: isize) {
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