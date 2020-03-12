use std::fmt;
use std::sync::Arc;

use crate::wait_group_impl::WaitGroupImpl;

pub struct GuardWaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl GuardWaitGroup {
    pub fn new() -> GuardWaitGroup {
        GuardWaitGroup {
            inner: Arc::new(WaitGroupImpl::new()),
        }
    }

    pub fn wait(&self) {
        self.inner.wait();
    }

    pub fn counter(&self) -> usize {
        self.inner.counter()
    }

    fn increment_counter(&self) {
        self.inner.increment();
    }

    fn done(&self) {
        self.inner.done();
    }

    unsafe fn inner(&self) -> Arc<WaitGroupImpl> {
        Arc::clone(&self.inner)
    }

    //todo add analogue for unique_doer() in SmartWaitGroup
    //todo smth like `pub fn clone_unique(&self) -> Option(Self)`
}

impl Clone for GuardWaitGroup {
    fn clone(&self) -> Self {
        let wg = GuardWaitGroup {
            inner: Arc::clone(&self.inner),
        };
        wg.increment_counter();
        wg
    }
}

impl Drop for GuardWaitGroup {
    fn drop(&mut self) {
        if let None = Arc::get_mut(&mut self.inner) {
            self.done();
        }
    }
}

impl fmt::Debug for GuardWaitGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}
