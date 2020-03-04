use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use wait_group::{GoWaitGroup, SmartWaitGroup, WaitGroup};

const ATTEMPTS: usize = 100; // number of attempts for searching deadlocks
const THREADS_NUMBER: usize = 100;
const INITIAL_VALUE: i32 = 0;
const EXPECTED_AFTER_WAITING: i32 = 100;
const EXPECTED_AFTER_JOINING: i32 = -1;
const INCORRECT_THREADS_NUMBER: usize = THREADS_NUMBER - 1;

#[test]
fn wait_group() {
    for _ in 0..ATTEMPTS {
        let counter = Arc::new(AtomicI32::new(INITIAL_VALUE));

        let wg = WaitGroup::new();

        // Spawn N threads and increment the counter
        let thread_handlers = (0..THREADS_NUMBER)
            .map(|_| {
                let _unexpected_clone_of_wg = wg.clone();
                let wg = wg.clone();
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    drop(wg)
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        wg.wait();
        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_WAITING);
        counter.store(EXPECTED_AFTER_JOINING, Ordering::SeqCst);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_JOINING);
    }

    // Methods increment_counter() and done() are private,
    // so we can't invalidate invariants for inner counter.
    // Instead, we are using clone() and drop();
}

#[test]
fn smart_wg() {
    for _ in 0..ATTEMPTS {
        let counter = Arc::new(AtomicI32::new(INITIAL_VALUE));

        let wg = SmartWaitGroup::new();
        let waiter = wg.waiter();
        // Spawn N threads and set flag to false;
        let thread_handlers = (0..THREADS_NUMBER)
            .map(|_| {
                let _unexpected_doer = wg.doer();
                let doer = wg.doer();
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    drop(doer)
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        waiter.wait();
        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_WAITING);
        counter.store(EXPECTED_AFTER_JOINING, Ordering::SeqCst);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_JOINING);
    }
}

#[test]
fn go_wg() {
    for _ in 0..ATTEMPTS {
        let counter = Arc::new(AtomicI32::new(INITIAL_VALUE));

        let wg = GoWaitGroup::new();
        wg.add(THREADS_NUMBER as isize);
        // Spawn N threads and set flag to false;
        let thread_handlers = (0..THREADS_NUMBER)
            .map(|_| {
                let wg = wg.clone();
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    wg.done();
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        wg.wait();
        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_WAITING);
        counter.store(EXPECTED_AFTER_JOINING, Ordering::SeqCst);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_JOINING);
    }
}

#[test]
#[should_panic]
fn go_wg_negative_counter() {
    for _ in 0..ATTEMPTS {
        let counter = Arc::new(AtomicI32::new(INITIAL_VALUE));
        let wg = GoWaitGroup::new();
        wg.add(INCORRECT_THREADS_NUMBER as isize);
        // Spawn N threads and set flag to false;
        let thread_handlers = (0..THREADS_NUMBER)
            .map(|_| {
                let wg = wg.clone();
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    wg.done();
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        wg.wait();

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap(); //panic on the last thread here
        }
        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_WAITING);
        counter.store(EXPECTED_AFTER_JOINING, Ordering::SeqCst);
        assert_eq!(counter.load(Ordering::SeqCst), EXPECTED_AFTER_JOINING);
    }
}
