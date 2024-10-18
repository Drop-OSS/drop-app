use crate::downloads::progress::ProgressChecker;

#[test]
fn test_progress_sequentially() {
    let p = ProgressChecker::new(Box::new(test_fn));
    p.run_contexts_sequentially((1..100).collect());
    println!("Progress: {}", p.get_progress_percentage(100));
}
#[test]
fn test_progress_parallel() {
    let p = ProgressChecker::new(Box::new(test_fn));
    p.run_contexts_parallel((1..100).collect(), 10);
}

fn test_fn(int: usize) {
    println!("{}", int);
}