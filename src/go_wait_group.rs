use crate::wait_group_impl::{WaitGroupImpl, WaitGroupError};
use std::sync::Arc;

#[derive(Clone)]
pub struct GoWaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl GoWaitGroup {
    pub fn new() -> Self {
        GoWaitGroup { inner: Arc::new(WaitGroupImpl::new()) }
    }

    pub fn wait(&self) {
        self.inner.wait();
    }

    #[must_use]
    pub fn try_wait(&self) -> Result<(), WaitGroupError>{
        self.inner.try_wait()
    }

    pub fn add(&self, delta: isize) {
        self.inner.add_count_unchecked(delta);
    }

    pub fn done(&self) {
        self.inner.done();
    }
}