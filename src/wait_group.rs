use std::fmt;
use std::sync::Arc;

use crate::wait_group_impl::WaitGroupImpl;
use crate::WaitGroupError;

pub struct WaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl WaitGroup {
    pub fn new() -> WaitGroup {
        WaitGroup {
            inner: Arc::new(WaitGroupImpl::new()),
        }
    }

    pub fn wait(&self) {
        self.inner.wait();
    }

    fn increment_counter(&self) {
        self.inner.increment_count();
    }

    fn done(&self) {
        self.inner.done();
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        let wg = WaitGroup {
            inner: Arc::clone(&self.inner),
        };
        wg.increment_counter();
        wg
    }
}

impl Drop for WaitGroup {
    fn drop(&mut self) {
        if let None = Arc::get_mut(&mut self.inner) {
            self.done();
        }
    }
}

impl fmt::Debug for WaitGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}
