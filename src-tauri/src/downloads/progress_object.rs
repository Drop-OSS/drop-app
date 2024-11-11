use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

#[derive(Clone)]
pub struct ProgressObject {
    max: usize,
    progress_instances: Arc<Vec<Arc<AtomicUsize>>>,
}

impl ProgressObject {
    pub fn new(max: usize, length: usize) -> Self {
        let arr = (0..length).map(|_| { Arc::new(AtomicUsize::new(0)) }).collect();
        Self {
            max,
            progress_instances: Arc::new(arr)
        }
    }
    pub fn sum(&self) -> usize {
        self.progress_instances.iter().map(|instance| {
            instance.load(Ordering::Relaxed)
        }).sum()
    }

    pub fn get_progress(&self) -> f64 {
        self.sum() as f64 / self.max as f64
    }
    pub fn get(&self, index: usize) -> Arc<AtomicUsize> {
        self.progress_instances[index].clone()
    }
}