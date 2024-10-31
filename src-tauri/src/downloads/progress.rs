use log::info;
use rayon::ThreadPoolBuilder;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ProgressChecker<T>
where
    T: 'static + Send + Sync,
{
    counter: Arc<AtomicUsize>,
    f: Arc<Box<dyn Fn(T, Arc<AtomicBool>) + Send + Sync + 'static>>,
    callback: Arc<AtomicBool>
}

impl<T> ProgressChecker<T>
where
    T: Send + Sync,
{
    pub fn new(
        f: Box<dyn Fn(T, Arc<AtomicBool>) + Send + Sync + 'static>,
        counter_reference: Arc<AtomicUsize>,
        callback: Arc<AtomicBool>
    ) -> Self {
        Self {
            f: f.into(),
            counter: counter_reference,
            callback
        }
    }
    pub fn run_contexts_sequentially(&self, contexts: Vec<T>) {
        for context in contexts {
            (self.f)(context, self.callback.clone());
            self.counter.fetch_add(1, Ordering::Release);
        }
    }
    pub fn run_contexts_parallel_background(&self, contexts: Vec<T>, max_threads: usize) {
        let threads = ThreadPoolBuilder::new()
            // If max_threads == 0, then the limit will be determined
            // by Rayon's internal RAYON_NUM_THREADS
            .num_threads(max_threads)
            .build()
            .unwrap();

        for context in contexts {
            let callback = self.callback.clone();
            let f = self.f.clone();
            threads.spawn(move || f(context, callback));
        }
    }
    pub fn run_context_parallel(&self, contexts: Vec<T>, max_threads: usize) {
        let threads = ThreadPoolBuilder::new()
            .num_threads(max_threads)
            .build()
            .unwrap();

        threads.scope(|s| {
            for context in contexts {
                let callback = self.callback.clone();
                let f = self.f.clone();
                s.spawn(move |_| {info!("Running thread"); f(context, callback)});
            }
        });
        info!("Concluded scope");
        
    }
    pub fn get_progress(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
    // I strongly dislike type casting in my own code, so I've shovelled it into here
    pub fn get_progress_percentage<C: Into<f64>>(&self, capacity: C) -> f64 {
        (self.get_progress() as f64) / (capacity.into())
    }
}
