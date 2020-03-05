use std::sync::Arc;

use crate::wait_group_impl::WaitGroupImpl;

#[derive(Clone)]
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

    pub fn unique_doer(&self) -> Option<Doer> {
        Doer::unique(Arc::clone(&self.inner))
    }

    pub fn waiter(&self) -> Waiter {
        Waiter::new(Arc::clone(&self.inner))
    }

    pub fn counter(&self) -> usize {
        self.inner.counter()
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

    fn unique(wait_group: Arc<WaitGroupImpl>) -> Option<Self> {
        if wait_group.increment_if_empty() {
            Some(Doer { wait_group })
        }
        else {
            None
        }
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

// todo come up with a good name
// todo impl MultiWaitGroup
// todo impl try_switch()
pub fn switch(first: &SmartWaitGroup, second: &SmartWaitGroup) -> Doer {
    // Ensure that first and second are differ (not an identical allocations)
    // for avoiding deadlock
    assert!(!Arc::ptr_eq(&first.inner, &second.inner));
    let doer = first.doer();
    second.waiter().wait();
    doer
}

// todo come up with a good name
// todo impl MultiWaitGroup
// todo impl try_switch_unique()
pub fn switch_unique(first: &SmartWaitGroup, second: &SmartWaitGroup) -> Option<Doer> {
    // Ensure that first and second are differ (not an identical allocations)
    // for avoiding deadlock
    assert!(!Arc::ptr_eq(&first.inner, &second.inner));
    let doer = first.unique_doer();
    if let Some(_) = doer {
        second.waiter().wait();
    }
    doer
}