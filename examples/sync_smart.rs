use wait_group::{SmartWaitGroup, Doer, Order};

use rayon::ThreadPoolBuilder;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;

struct Context {
    resource_counter: AtomicUsize,
    normal_wg: SmartWaitGroup, //WaitGroup for normal task
    special_wg: SmartWaitGroup, //WaitGroup for special task
    scope_wg: SmartWaitGroup, //WaitGroup for global scope
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

fn normal_task(c: Arc<Context> ) {
    {
        let normal_dor = c.normal_wg.switch_wait_do(&c.special_wg);
        c.resource_counter.fetch_add(1, Ordering::SeqCst);
    }
    if c.resource_counter.load(Ordering::SeqCst) >= 60 {
        if let Some(doer) = c.special_wg.switch_unique(&c.normal_wg) {
            special_task(c, doer);
        }
    }
}

fn special_task(c: Arc<Context>, doer: Doer) {
    c.resource_counter.store(0, Ordering::SeqCst);
}

fn task(c: Arc<Context>) {
    let scope_doer = c.scope_wg.doer();
    normal_task(c);
}

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

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