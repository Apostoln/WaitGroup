use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use std::thread;

use wait_group::GuardWaitGroup;

fn process_counter(counter: Arc<AtomicIsize>, _wg: GuardWaitGroup) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(_wg) implicit call
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
    println!("{}", counter.load(Ordering::SeqCst)); //100
}
