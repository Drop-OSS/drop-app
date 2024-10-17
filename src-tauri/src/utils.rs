use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use crate::utils::ProgressChecker::Complete;

#[derive(Eq, PartialEq)]
pub enum ProgressChecker {
    Complete,
    Incomplete
}

// This function is designed to take in any function which does not regularly return a value,
// and instead loops over it until it returns "Complete". The current number of iterations
// is counted by "progress"
pub async fn progress_updater(function: Box<dyn Fn() -> ProgressChecker>, progress: AtomicUsize) {
    loop {
        if function() == ProgressChecker::Complete { break }
        progress.fetch_add(1, Relaxed);
    }
}

pub async fn threaded_progress_updater<F>(f: F, progress: AtomicUsize, max_threads: usize, instances: usize) -> ProgressChecker
where F: Fn() -> ProgressChecker + Send + Clone + Copy + 'static
{
    let mut threads = Vec::new();
    for instance in 0..instances {
        let func = tokio::spawn(async move {
            let res = f();
            return res
        });
        threads.push(func);
    }
    let mut completed = ProgressChecker::Incomplete;
    for thread in threads {
        if thread.await.unwrap() == Complete {
            completed = Complete
        }
        progress.fetch_add(1, Ordering::Relaxed);
    }
    completed
}

fn test() -> ProgressChecker {
    ProgressChecker::Incomplete
}