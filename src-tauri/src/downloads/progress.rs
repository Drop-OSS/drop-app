use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::ThreadPoolBuilder;

pub struct ProgressChecker<T>
where T: 'static + Send + Sync
{
    counter: Arc<AtomicUsize>,
    f: Arc<Box<dyn Fn(T) + Send + Sync + 'static>>,
}

impl<T> ProgressChecker<T>
where T: Send + Sync
{
    pub fn new(f: Box<dyn Fn(T) + Send + Sync + 'static>, counter_reference: Arc<AtomicUsize>) -> Self {
        Self {
            f: f.into(),
            counter: counter_reference
        }
    }
    pub async fn run_contexts_sequentially_async(&self, contexts: Vec<T>) {
        for context in contexts {
            (self.f)(context);
            self.counter.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn run_contexts_sequentially(&self, contexts: Vec<T>) {
        for context in contexts {
            (self.f)(context);
            self.counter.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn run_contexts_parallel(&self, contexts: Vec<T>, max_threads: usize) {
        let threads = ThreadPoolBuilder::new()
            // If max_threads == 0, then the limit will be determined
            // by Rayon's internal RAYON_NUM_THREADS
            .num_threads(max_threads)
            .build()
            .unwrap();

        for context in contexts {
            let f = self.f.clone();
            threads.spawn(move || f(context));
        }
    }
    pub fn get_progress(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
    // I strongly dislike type casting in my own code, so I've shovelled it into here
    pub fn get_progress_percentage<C: Into<f64>>(&self, capacity: C) -> f64 {
        (self.get_progress() as f64) / (capacity.into())
    }
}