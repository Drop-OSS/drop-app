use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
pub struct RollingProgressWindow<const S: usize> {
    window: Arc<[AtomicUsize; S]>,
    current: Arc<AtomicUsize>,
}
impl<const S: usize> RollingProgressWindow<S> {
    pub fn new() -> Self {
        Self {
            window: Arc::new([(); S].map(|()| AtomicUsize::new(0))),
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
        let valid = self
            .window
            .iter()
            .enumerate()
            .filter(|(i, _)| i < &current)
            .map(|(_, x)| x.load(Ordering::Acquire))
            .collect::<Vec<usize>>();
        let amount = valid.len();
        let sum = valid.into_iter().sum::<usize>();
        
        sum / amount
    }
    pub fn reset(&self) {
        self.window
            .iter()
            .for_each(|x| x.store(0, Ordering::Release));
        self.current.store(0, Ordering::Release);
    }
}
