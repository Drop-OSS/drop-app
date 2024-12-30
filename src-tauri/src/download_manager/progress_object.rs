use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
        Arc, Mutex, RwLock,
    },
    time::{Duration, Instant},
};

use log::info;

use super::download_manager::DownloadManagerSignal;

#[derive(Clone)]
pub struct ProgressObject {
    max: Arc<Mutex<usize>>,
    progress_instances: Arc<Mutex<Vec<Arc<AtomicUsize>>>>,
    start: Arc<Mutex<Instant>>,
    sender: Sender<DownloadManagerSignal>,

    points_towards_update: Arc<AtomicUsize>,
    points_to_push_update: Arc<AtomicUsize>,
    last_update: Arc<RwLock<Instant>>,
    amount_last_update: Arc<AtomicUsize>,
}

pub struct ProgressHandle {
    progress: Arc<AtomicUsize>,
    progress_object: Arc<ProgressObject>,
}

impl ProgressHandle {
    pub fn new(progress: Arc<AtomicUsize>, progress_object: Arc<ProgressObject>) -> Self {
        Self {
            progress,
            progress_object,
        }
    }
    pub fn set(&self, amount: usize) {
        self.progress.store(amount, Ordering::Relaxed);
    }
    pub fn add(&self, amount: usize) {
        self.progress
            .fetch_add(amount, std::sync::atomic::Ordering::Relaxed);
        self.progress_object.check_push_update(amount);
    }
}

static PROGRESS_UPDATES: usize = 100;

impl ProgressObject {
    pub fn new(max: usize, length: usize, sender: Sender<DownloadManagerSignal>) -> Self {
        let arr = Mutex::new((0..length).map(|_| Arc::new(AtomicUsize::new(0))).collect());
        // TODO: consolidate this calculation with the set_max function below
        let points_to_push_update = max / PROGRESS_UPDATES;
        Self {
            max: Arc::new(Mutex::new(max)),
            progress_instances: Arc::new(arr),
            start: Arc::new(Mutex::new(Instant::now())),
            sender,

            points_towards_update: Arc::new(AtomicUsize::new(0)),
            points_to_push_update: Arc::new(AtomicUsize::new(points_to_push_update)),
            last_update: Arc::new(RwLock::new(Instant::now())),
            amount_last_update: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn check_push_update(&self, amount_added: usize) {
        let current_amount = self
            .points_towards_update
            .fetch_add(amount_added, Ordering::Relaxed);

        let to_update = self.points_to_push_update.fetch_add(0, Ordering::Relaxed);

        if current_amount >= to_update {
            self.points_towards_update
                .fetch_sub(to_update, Ordering::Relaxed);
            self.sender
                .send(DownloadManagerSignal::UpdateUIQueue)
                .unwrap();
        }

        let last_update = self.last_update.read().unwrap();
        let last_update_difference = Instant::now().duration_since(*last_update).as_millis();
        if last_update_difference > 1000 {
            // push update
            drop(last_update);
            let mut last_update = self.last_update.write().unwrap();
            *last_update = Instant::now();
            drop(last_update);

            let current_amount = self.sum();
            let max = self.get_max();
            let amount_at_last_update = self.amount_last_update.fetch_add(0, Ordering::Relaxed);
            self.amount_last_update
                .store(current_amount, Ordering::Relaxed);

            let amount_since_last_update = current_amount - amount_at_last_update;

            let kilobytes_per_second = amount_since_last_update / (last_update_difference as usize).max(1);

            let remaining = max - current_amount; // bytes
            let time_remaining = (remaining / 1000) / kilobytes_per_second.max(1);
            self.sender
                .send(DownloadManagerSignal::UpdateUIStats(
                    kilobytes_per_second,
                    time_remaining,
                ))
                .unwrap();
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
        *self.max.lock().unwrap() = new_max;
        self.points_to_push_update
            .store(new_max / PROGRESS_UPDATES, Ordering::Relaxed);
        info!("points to push update: {}", new_max / PROGRESS_UPDATES);
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
