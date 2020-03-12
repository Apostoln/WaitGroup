pub use manual_wait_group::ManualWaitGroup;
pub use smart_wait_group::{SmartWaitGroup, Doer, Waiter, Order};
pub use wait_group::WaitGroup;
pub use wait_group_error::{Result, WaitGroupError};

mod manual_wait_group;
mod smart_wait_group;
mod wait_group;
mod wait_group_error;
mod wait_group_impl;