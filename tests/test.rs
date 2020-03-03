use std::thread;
use wait_group::{WaitGroup, SmartWaitGroup, GoWaitGroup};

const ATTEMPTS: usize = 100;

#[test]
fn wait_group() {
    for _ in 0..ATTEMPTS {
        let mut flag = None; //bool

        let wg = WaitGroup::new();
        const N: u64 = 100;

        // Spawn N threads and set flag to false;
        let thread_handlers = (0..N)
            .map(|_| {
                let wg = wg.clone();
                thread::spawn(move || {
                    flag = Some(false);
                    drop(wg)
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        wg.wait();
        flag = Some(true);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(flag, Some(true));
    }

    // Methods increment_counter() and done() are private,
    // so we can't invalidate invariants for inner counter.
    // Instead, we are using clone() and drop();
}

#[test]
fn smart_wait_group() {
    for _ in 0..ATTEMPTS {
        let mut flag = None; //bool

        let wg = SmartWaitGroup::new();
        let waiter = wg.waiter();
        const N: u64 = 100;
        // Spawn N threads and set flag to false;
        let thread_handlers = (0..N)
            .map(|_| {
                let doer = wg.doer();
                thread::spawn(move || {
                    flag = Some(false);
                    drop(doer)
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        waiter.wait();
        flag = Some(true);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(flag, Some(true));
    }
}

#[test]
fn go_wait_group() {
    for _ in 0..ATTEMPTS {
        let mut flag = None; //bool

        let wg = GoWaitGroup::new();
        const N: u64 = 100;
        wg.add(N as usize);

        // Spawn N threads and set flag to false;
        let thread_handlers = (0..N)
            .map(|_| {
                let wg = wg.clone();
                thread::spawn(move || {
                    flag = Some(false);
                    wg.done();
                })
            })
            .collect::<Vec<_>>();

        // Wait until all N threads are finished
        wg.wait();
        flag = Some(true);

        // Assure threads are finished for avoiding false-positive result
        for handler in thread_handlers {
            handler.join().unwrap();
        }

        assert_eq!(flag, Some(true));
    }
}