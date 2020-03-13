use wait_group::{Doer, SmartWaitGroup};

use rayon::ThreadPoolBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Context {
    resource_counter: AtomicUsize,
    normal_wg: SmartWaitGroup,  //WaitGroup for normal task
    special_wg: SmartWaitGroup, //WaitGroup for special task
}
impl Context {
    fn new() -> Self {
        Context {
            resource_counter: AtomicUsize::new(0),
            normal_wg: SmartWaitGroup::new(),
            special_wg: SmartWaitGroup::new(),
        }
    }
}

fn normal_task(c: Arc<Context>, _normal_doer: Doer) {
    c.resource_counter.fetch_add(1, Ordering::SeqCst);
    //drop(_normal_doer) implicit call
}


fn special_task(c: Arc<Context>, _special_doer: Doer) {
    c.resource_counter.store(0, Ordering::SeqCst);
    //drop(_special_doer) implicit call
}

fn task(c: Arc<Context>) {
    c.special_wg.waiter().wait();
    let normal_doer = c.normal_wg.doer();
    normal_task(Arc::clone(&c), normal_doer);

    if c.resource_counter.load(Ordering::SeqCst) >= 60 {
        if let Some(special_doer) = c.special_wg.unique_doer() {
            c.normal_wg.waiter().wait();
            special_task(Arc::clone(&c), special_doer);
        }
    }
}

fn main() {
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();

    let context = Arc::new(Context::new());

    pool.scope( |s| {
        for _ in 0..100 {
            let context = context.clone();
            s.spawn(|_| task(context));
        }
    });


    println!("{}", context.resource_counter.load(Ordering::SeqCst));
}