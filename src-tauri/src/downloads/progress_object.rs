use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

#[derive(Clone)]
pub struct ProgressObject {
    max: Arc<Mutex<usize>>,
    progress_instances: Arc<Mutex<Vec<Arc<AtomicUsize>>>>,
    start: Arc<Mutex<Instant>>,
}

impl ProgressObject {
    pub fn new(max: usize, length: usize) -> Self {
        let arr = Mutex::new((0..length).map(|_| Arc::new(AtomicUsize::new(0))).collect());
        Self {
            max: Arc::new(Mutex::new(max)),
            progress_instances: Arc::new(arr),
            start: Arc::new(Mutex::new(Instant::now())),
        }
    }
    pub fn set_time_now(&self) {
        *self.start.lock().unwrap() = Instant::now();
    }
    pub fn sum(&self) -> usize {
        self.progress_instances
            .lock()
            .unwrap()
            .iter()
            .map(|instance| instance.load(Ordering::Relaxed))
            .sum()
    }
    pub fn get_max(&self) -> usize {
        *self.max.lock().unwrap()
    }
    pub fn set_max(&self, new_max: usize) {
        *self.max.lock().unwrap() = new_max
    }
    pub fn set_size(&self, length: usize) {
        *self.progress_instances.lock().unwrap() =
            (0..length).map(|_| Arc::new(AtomicUsize::new(0))).collect();
    }

    pub fn get_progress(&self) -> f64 {
        self.sum() as f64 / self.get_max() as f64
    }
    pub fn get(&self, index: usize) -> Arc<AtomicUsize> {
        self.progress_instances.lock().unwrap()[index].clone()
    }
}
