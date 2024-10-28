use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use crate::downloads::progress::ProgressChecker;

#[test]
fn test_progress_sequentially() {
    let counter = Arc::new(AtomicUsize::new(0));
    let p = ProgressChecker::new(Box::new(test_fn), counter.clone());
    p.run_contexts_sequentially((1..100).collect());
    println!("Progress: {}", p.get_progress_percentage(100));
}
#[test]
fn test_progress_parallel() {
    let counter = Arc::new(AtomicUsize::new(0));
    let p = ProgressChecker::new(Box::new(test_fn), counter.clone());
    p.run_contexts_parallel_background((1..100).collect(), 10);
}

fn test_fn(int: usize) {
    println!("{}", int);
}