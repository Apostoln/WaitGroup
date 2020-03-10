use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use wait_group::GoWaitGroup;

fn process_counter(counter: Arc<AtomicIsize>, wg: GoWaitGroup) {
    counter.fetch_add(1, Ordering::SeqCst);
    wg.done();
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));

    let wg = GoWaitGroup::new();
    wg.add(100);

    // Spawn 100 threads and set flag to false;
    for _ in 0..100 {
        let wg = wg.clone();
        let counter = Arc::clone(&counter);
        thread::spawn(move || process_counter(counter, wg));
    }

    // Wait until all 100 threads are finished
    wg.wait();
    println!("{}", counter.load(Ordering::SeqCst)); //100
}