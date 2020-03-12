use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use wait_group::WaitGroup;

fn process_counter(counter: Arc<AtomicIsize>, wg: WaitGroup) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(wg) implicit call
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));

    let wg = WaitGroup::new();

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