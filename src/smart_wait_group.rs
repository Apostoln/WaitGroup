use std::sync::Arc;

use crate::wait_group_impl::WaitGroupImpl;
use crate::WaitGroupError;

pub struct SmartWaitGroup {
    inner: Arc<WaitGroupImpl>,
}
impl SmartWaitGroup {
    pub fn new() -> Self {
        SmartWaitGroup {
            inner: Arc::new(WaitGroupImpl::new()),
        }
    }

    pub fn doer(&self) -> Doer {
        Doer::new(Arc::clone(&self.inner))
    }

    pub fn waiter(&self) -> Waiter {
        Waiter::new(Arc::clone(&self.inner))
    }

    unsafe fn inner(&self) -> Arc<WaitGroupImpl> {
        Arc::clone(&self.inner)
    }
}

pub struct Doer {
    wait_group: Arc<WaitGroupImpl>,
}
impl Doer {
    fn new(wait_group: Arc<WaitGroupImpl>) -> Self {
        wait_group.increment();
        Doer { wait_group }
    }

    fn done(&self) {
        self.wait_group.done();
    }
}
impl Drop for Doer {
    fn drop(&mut self) {
        self.done();
    }
}

impl Clone for Doer {
    fn clone(&self) -> Self {
        Doer::new(Arc::clone(&self.wait_group))
    }
}

pub struct Waiter {
    wait_group: Arc<WaitGroupImpl>,
}
impl Waiter {
    fn new(wait_group: Arc<WaitGroupImpl>) -> Self {
        Waiter { wait_group }
    }

    pub fn wait(&self) {
        self.wait_group.wait();
    }
}
impl Clone for Waiter {
    fn clone(&self) -> Self {
        Waiter::new(Arc::clone(&self.wait_group))
    }
}
