use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use std::thread;

use wait_group::{Doer, Waiter, SmartWaitGroup};
use std::thread::sleep;
use std::time::Duration;

fn process_counter(counter: Arc<AtomicIsize>, _doer: Doer) {
    counter.fetch_add(1, Ordering::SeqCst);
    //drop(_doer) implicit call
}

fn heavy_process_counter(counter: Arc<AtomicIsize>, wg: Waiter) {
    let delta = {
        sleep(Duration::from_secs(1)); //emulation of some heavy preparing
        42
    };
    wg.wait(); //deadlock
    eprintln!("{}", counter.load(Ordering::SeqCst) + delta); //142
}

fn spawn_process_threads(counter: Arc<AtomicIsize>, doer: Doer) {
    for _ in 0..100 {
        let doer = doer.clone();
        let counter = Arc::clone(&counter);
        thread::spawn(move || process_counter(counter, doer));
    }
}

fn spawn_heavy_process_thread(counter: Arc<AtomicIsize>, waiter: Waiter) {
    let counter = Arc::clone(&counter);
    thread::spawn(move || heavy_process_counter(counter, waiter));
}

fn main() {
    let counter = Arc::new(AtomicIsize::new(0));
    let (waiter, doer) = SmartWaitGroup::splitted();
    spawn_process_threads(Arc::clone(&counter), doer);
    spawn_heavy_process_thread(Arc::clone(&counter), waiter.clone());
    waiter.wait();
    println!("{}", counter.load(Ordering::SeqCst)); //100
}