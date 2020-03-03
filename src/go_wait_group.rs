use crate::wait_group_impl::WaitGroupImpl;
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

    pub fn add(&self, delta: usize) {
        self.inner.add_count(delta);
    }

    pub fn done(&self) {
        self.inner.done();
    }
}