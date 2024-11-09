/*
use atomic_counter::RelaxedCounter;

use crate::downloads::progress::ProgressChecker;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;


#[test]
fn test_progress_sequentially() {
    let counter = Arc::new(RelaxedCounter::new(0));
    let callback = Arc::new(AtomicBool::new(false));
    let p = ProgressChecker::new(Box::new(test_fn), counter.clone(), callback, 100);
    p.run_contexts_sequentially((1..100).collect());
    println!("Progress: {}", p.get_progress_percentage());
}
#[test]
fn test_progress_parallel() {
    let counter = Arc::new(RelaxedCounter::new(0));
    let callback = Arc::new(AtomicBool::new(false));
    let p = ProgressChecker::new(Box::new(test_fn), counter.clone(), callback, 100);
    p.run_contexts_parallel_background((1..100).collect(), 10);
}

fn test_fn(int: usize, _callback: Arc<AtomicBool>, _counter: Arc<RelaxedCounter>) {
    println!("{}", int);
}

*/
