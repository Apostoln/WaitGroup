use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use wait_group::{SmartWaitGroup, Doer, Waiter};

fn process_counter(counter: Arc<AtomicIsize>, doer: Doer) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(doer) implicit call
}

fn spawn_threads(counter: Arc<AtomicIsize>, doer: Doer) {
    for _ in 0..100 {
        let doer = doer.clone();
        let counter = Arc::clone(&counter);
        thread::spawn(move || process_counter(counter, doer));
    }
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));

    let (waiter, doer) = SmartWaitGroup::splitted();

    // Spawn 100 threads and increment the counter
    spawn_threads(Arc::clone(&counter), doer);

    // Wait until all 100 threads are finished
    waiter.wait();
    println!("{}", counter.load(Ordering::SeqCst)); //100
}