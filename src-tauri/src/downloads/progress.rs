use atomic_counter::{AtomicCounter, RelaxedCounter};
use log::info;
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex, RwLock};

use super::download_agent::GameDownloadState;

pub struct ProgressChecker<T>
where
    T: 'static + Send + Sync,
{
    counter: Arc<RelaxedCounter>,
    f: Arc<
        Box<dyn Fn(T, Arc<RwLock<GameDownloadState>>, Arc<RelaxedCounter>) + Send + Sync + 'static>,
    >,
    status: Arc<RwLock<GameDownloadState>>,
    capacity: Mutex<usize>,
}

impl<T> ProgressChecker<T>
where
    T: Send + Sync,
{
    pub fn new(
        f: Box<
            dyn Fn(T, Arc<RwLock<GameDownloadState>>, Arc<RelaxedCounter>) + Send + Sync + 'static,
        >,
        counter: Arc<RelaxedCounter>,
        status: Arc<RwLock<GameDownloadState>>,
        capacity: usize,
    ) -> Self {
        Self {
            f: f.into(),
            counter,
            status,
            capacity: capacity.into(),
        }
    }
    pub fn run_context_parallel(&self, contexts: Vec<T>, max_threads: usize) {
        let threads = ThreadPoolBuilder::new()
            .num_threads(max_threads)
            .build()
            .unwrap();

        threads.scope(|s| {
            for context in contexts {
                let status = self.status.clone();
                let counter = self.counter.clone();
                let f = self.f.clone();
                s.spawn(move |_| {
                    info!("Running thread");
                    f(context, status, counter)
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
