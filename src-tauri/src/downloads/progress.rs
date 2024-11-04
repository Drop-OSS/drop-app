use atomic_counter::{AtomicCounter, RelaxedCounter};
use log::info;
use rayon::ThreadPoolBuilder;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct ProgressChecker<T>
where
    T: 'static + Send + Sync,
{
    counter: Arc<RelaxedCounter>,
    f: Arc<Box<dyn Fn(T, Arc<AtomicBool>, Arc<RelaxedCounter>) + Send + Sync + 'static>>,
    callback: Arc<AtomicBool>,
    capacity: Mutex<usize>,
}

impl<T> ProgressChecker<T>
where
    T: Send + Sync,
{
    pub fn new(
        f: Box<dyn Fn(T, Arc<AtomicBool>, Arc<RelaxedCounter>) + Send + Sync + 'static>,
        counter: Arc<RelaxedCounter>,
        callback: Arc<AtomicBool>,
        capacity: usize,
    ) -> Self {
        Self {
            f: f.into(),
            counter,
            callback,
            capacity: capacity.into(),
        }
    }
    pub fn run_contexts_sequentially(&self, contexts: Vec<T>) {
        for context in contexts {
            (self.f)(context, self.callback.clone(), self.counter.clone());
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
            let counter = self.counter.clone();
            let f = self.f.clone();
            threads.spawn(move || f(context, callback, counter));
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
                let counter = self.counter.clone();
                let f = self.f.clone();
                s.spawn(move |_| {
                    info!("Running thread");
                    f(context, callback, counter)
                });
            }
        });
        info!("Concluded scope");
    }
    pub fn set_capacity(&self, capacity: usize) {
        let mut lock = self.capacity.lock().unwrap();
        *lock = capacity;
    }
    pub fn get_progress(&self) -> usize {
        self.counter.get()
    }
    // I strongly dislike type casting in my own code, so I've shovelled it into here
    pub fn get_progress_percentage(&self) -> f64 {
        (self.get_progress() as f64) / (*self.capacity.lock().unwrap() as f64)
    }
}
