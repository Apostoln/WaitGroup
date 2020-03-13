use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use std::thread;

use wait_group::GuardWaitGroup;
use std::thread::sleep;
use std::time::Duration;

fn process_counter(counter: Arc<AtomicIsize>, _wg: GuardWaitGroup) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(_wg) implicit call
}

fn heavy_process_counter(counter: Arc<AtomicIsize>, wg: GuardWaitGroup) {
    let delta = {
        sleep(Duration::from_secs(1)); //emulation of some heavy preparing
        42
    };
    wg.wait(); //deadlock
    eprintln!("{}", counter.load(Ordering::SeqCst) + delta)
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

    {
        // Spawn one more thread
        let wg = wg.clone();
        let counter = Arc::clone(&counter);
        thread::spawn(move || heavy_process_counter(counter, wg));
    }
    // Wait until all 100 threads are finished
    wg.wait();
    println!("{}", counter.load(Ordering::SeqCst));
}
