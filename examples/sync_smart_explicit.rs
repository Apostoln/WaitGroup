use wait_group::{Doer, SmartWaitGroup};

use rayon::ThreadPoolBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Context {
    resource_counter: AtomicUsize,
    normal_wg: SmartWaitGroup,  //WaitGroup for normal task
    special_wg: SmartWaitGroup, //WaitGroup for special task
    scope_wg: SmartWaitGroup,   //WaitGroup for global scope
}
impl Context {
    fn new() -> Self {
        Context {
            resource_counter: AtomicUsize::new(0),
            normal_wg: SmartWaitGroup::new(),
            special_wg: SmartWaitGroup::new(),
            scope_wg: SmartWaitGroup::new(),
        }
    }
}

fn normal_task(c: Arc<Context>) {
    {
        c.special_wg.waiter().wait();
        let _normal_doer = c.normal_wg.doer();
        c.resource_counter.fetch_add(1, Ordering::SeqCst);
    }
    if c.resource_counter.load(Ordering::SeqCst) >= 60 {
        if let Some(doer) = c.special_wg.unique_doer() {
            c.normal_wg.waiter().wait();
            special_task(c, doer);
        }
    }
}

fn special_task(c: Arc<Context>, _doer: Doer) {
    c.resource_counter.store(0, Ordering::SeqCst);
    //drop(_doer) implicit call
}

fn task(c: Arc<Context>) {
    let _scope_doer = c.scope_wg.doer();
    normal_task(c);
}

fn main() {
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();

    let context = Arc::new(Context::new());

    for _ in 0..100 {
        let context = context.clone();
        pool.install(|| {
            task(context);
        });
    }

    context.scope_wg.waiter().wait();

    println!("{}", context.resource_counter.load(Ordering::SeqCst));
}
