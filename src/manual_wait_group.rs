use crate::wait_group_impl::WaitGroupImpl;
use crate::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct ManualWaitGroup {
    inner: Arc<WaitGroupImpl>,
}

impl ManualWaitGroup {
    pub fn new() -> Self {
        ManualWaitGroup {
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
        self.inner.add(delta);
    }

    #[must_use]
    pub fn add_if_empty(&self, delta: usize) -> bool {
        self.inner.add_if_empty(delta)
    }

    #[must_use]
    pub fn try_done(&self) -> Result<()> {
        self.inner.try_done()
    }

    pub fn done(&self) {
        self.inner.done();
    }

    pub fn counter(&self) -> usize {
        self.inner.counter()
    }

    unsafe fn inner(&self) -> Arc<WaitGroupImpl> {
        Arc::clone(&self.inner)
    }
}
