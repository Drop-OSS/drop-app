use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct RollingProgressWindow<const S: usize> {
    window: Arc<[AtomicUsize; S]>,
    current: Arc<AtomicUsize>,
}
impl<const S: usize> RollingProgressWindow<S> {
    pub fn new() -> Self {
        Self {
            window: Arc::new([(); S].map(|_| AtomicUsize::new(0))),
            current: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn update(&self, kilobytes_per_second: usize) {
        let index = self.current.fetch_add(1, Ordering::SeqCst);
        let current = &self.window[index % S];
        current.store(kilobytes_per_second, Ordering::SeqCst);
    }
    pub fn get_average(&self) -> usize {
        let current = self.current.load(Ordering::SeqCst);
        self.window
            .iter()
            .enumerate()
            .filter(|(i, _)| i < &current)
            .map(|(_, x)| x.load(Ordering::Relaxed))
            .sum::<usize>()
            / S
    }
}
