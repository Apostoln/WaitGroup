use std::fmt;
use std::sync::{Arc, Mutex, Condvar};

pub struct WaitGroup(Arc<WaitGroupImpl>);

struct WaitGroupImpl {
    cond: Condvar,
    count: Mutex<usize>,
}

impl WaitGroup {
    pub fn new() -> WaitGroup {
        WaitGroup(Arc::new(WaitGroupImpl {
            cond: Condvar::new(),
            count: Mutex::new(0),
        }))
    }

    pub fn wait(&self) {
        let mut count = self.0.count.lock().unwrap();
        while *count > 0 {
            count = self.0.cond.wait(count).unwrap();
        }
    }

    fn increment_counter(&self) {
        let mut count = self.0.count.lock().unwrap();
        *count += 1;
        self.notify_if_empty(*count);
    }

    fn done(&self) {
        let mut count = self.0.count.lock().unwrap();
        if *count > 0 {
            *count -= 1;
            self.notify_if_empty(*count);
        }
    }

    fn notify_if_empty(&self, count: usize) {
        if count == 0 {
            self.0.cond.notify_all();
        }
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        let wg = WaitGroup(Arc::clone(&self.0));
        wg.increment_counter();
        wg
    }
}

impl Drop for WaitGroup {
    fn drop(&mut self) {
        self.done();
    }
}

impl fmt::Debug for WaitGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let count = self.0.count.lock().unwrap();
        write!(f, "WaitGroup {{ count {:?} }}", *count)
    }
}