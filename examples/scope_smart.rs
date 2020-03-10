use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use wait_group::{SmartWaitGroup, Doer, Waiter};

fn process_counter(counter: Arc<AtomicIsize>, wg: Doer) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(wg) implicit call
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));

    let wg = SmartWaitGroup::new();

    // Spawn N threads and increment the counter
    for _ in 0..100 {
        let doer = wg.doer();
        let counter = Arc::clone(&counter);
        thread::spawn(move || process_counter(counter, doer));
    }

    // Wait until all N threads are finished
    wg.waiter().wait();
    println!("{}", counter.load(Ordering::SeqCst)); //100
}