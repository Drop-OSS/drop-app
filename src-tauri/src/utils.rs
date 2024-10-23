

/*
// This function is designed to take in any function which does not regularly return a value,
// and instead loops over it until it returns "Complete". The current number of iterations
// is counted by "progress"
pub async fn progress_updater(function: Box<dyn Fn() -> ProgressChecker>, progress: AtomicUsize) {
    loop {
        if function() == ProgressChecker::Complete { break }
        progress.fetch_add(1, Relaxed);
    }
}

pub async fn new_progress_updater<T, D>(function: Box<dyn Fn(T) -> D>, contexts: T, progress: AtomicUsize) {
    
}

pub async fn threaded_progress_updater<F>(f: F, progress: AtomicUsize, max_threads: usize, instances: usize) -> ProgressChecker
where F: Fn() -> ProgressChecker + Send + Clone + Copy + 'static
{
    let mut threads = Vec::new();
    let pool = ThreadPoolBuilder::new().num_threads(max_threads).build().unwrap();
    for instance in 0..instances {
        pool.spawn(move || -> ProgressChecker {
            let res = f();
            return res
        });
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
 */