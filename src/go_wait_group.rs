use crate::wait_group_impl::WaitGroupImpl;
use crate::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct GoWaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl GoWaitGroup {
    pub fn new() -> Self {
        GoWaitGroup {
            inner: Arc::new(WaitGroupImpl::new()),
        }
    }

    pub fn wait(&self) {
        self.inner.wait();
    }

    #[must_use]
    pub fn try_add(&self, delta: isize) -> Result<()> {
        self.inner.try_add(delta)
    }

    pub fn add(&self, delta: isize) {
        self.try_add(delta).unwrap();
    }

    #[must_use]
    pub fn try_done(&self) -> Result<()> {
        self.inner.try_done()
    }

    pub fn done(&self) {
        self.try_done().unwrap();
    }

    pub fn counter(&self) -> usize {
        self.inner.counter()
    }

    unsafe fn inner(&self) -> Arc<WaitGroupImpl> {
        Arc::clone(&self.inner)
    }

    //todo add analogue for unique_doer() in SmartWaitGroup
}
