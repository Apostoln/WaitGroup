use std::convert::TryFrom;
use std::fmt;
use std::sync::{Condvar, Mutex};

use crate::{Result, WaitGroupError};

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

    pub fn increment_count(&self) {
        self.add_count_unchecked(1);
    }

    pub fn add_count(&self, delta: isize) {
        self.try_add_count(delta).unwrap();
    }

    pub fn try_add_count(&self, delta: isize) -> Result<()>{
        let mut count = self.counter.lock().unwrap();
        let res = *count as isize + delta;
        if res < 0 {
            Err(WaitGroupError::NegativeCounter(res))
        }
        else {
            *count = res as usize;
            self.notify_if_empty(*count);
            Ok(())
        }
    }

    pub fn add_count_unchecked(&self, delta: usize) {
        let mut count = self.counter.lock().unwrap();
        *count += delta;
    }

    pub fn try_done(&self) -> Result<()> {
        self.try_add_count(-1)
    }

    pub fn done(&self) {
        self.try_done().unwrap();
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