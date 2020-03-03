use std::fmt;
use std::sync::Arc;

use crate::wait_group_impl::WaitGroupImpl;

/// This synchronization primitive enables one thread to wait the others threads.
pub struct WaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl WaitGroup {
    pub fn new() -> WaitGroup {
        WaitGroup { inner: Arc::new(WaitGroupImpl::new()) }
    }

    pub fn wait(&self) {
        self.inner.wait();
    }

    fn increment_counter(&self) {
        self.inner.increment_counter();
    }

    fn done(&self) {
        self.inner.done();
    }

    pub unsafe fn raw_clone(&self) -> Self {
        WaitGroup { inner: Arc::clone(&self.inner) }
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        let wg = WaitGroup { inner: Arc::clone(&self.inner) };
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
        self.fmt(f)
    }
}