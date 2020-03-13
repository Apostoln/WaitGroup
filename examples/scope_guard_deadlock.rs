use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use std::thread;

use wait_group::GuardWaitGroup;

fn some_condition() -> bool {
    true
}

fn process_counter(counter: Arc<AtomicIsize>, wg: GuardWaitGroup) {
    counter.fetch_add(1, Ordering::SeqCst);
    if some_condition() {
        wg.wait(); //deadlock
    }
    //drop(wg) implicit call
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));

    let wg = GuardWaitGroup::new();

    // Spawn 100 threads and process the counter
    for _ in 0..100 {
        let wg = wg.clone();
        let counter = Arc::clone(&counter);
        thread::spawn(move || process_counter(counter, wg));
    }

    // Wait until all 100 threads are finished
    wg.wait();
    println!("{}", counter.load(Ordering::SeqCst));
}
