use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
    },
    time::{Duration, Instant},
};

use atomic_instant_full::AtomicInstant;
use throttle_my_fn::throttle;
use utils::{lock, send};

use crate::download_manager_frontend::DownloadManagerSignal;

use super::rolling_progress_updates::RollingProgressWindow;

#[derive(Clone, Debug)]
pub struct ProgressObject {
    max: Arc<Mutex<usize>>,
    progress_instances: Arc<Mutex<Vec<Arc<AtomicUsize>>>>,
    start: Arc<Mutex<Instant>>,
    sender: Sender<DownloadManagerSignal>,
    //last_update: Arc<RwLock<Instant>>,
    last_update_time: Arc<AtomicInstant>,
    bytes_last_update: Arc<AtomicUsize>,
    rolling: RollingProgressWindow<1000>,
}

#[derive(Clone)]
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
        self.progress.store(amount, Ordering::Release);
    }
    pub fn add(&self, amount: usize) {
        self.progress
            .fetch_add(amount, std::sync::atomic::Ordering::AcqRel);
        calculate_update(&self.progress_object);
    }
    pub fn skip(&self, amount: usize) {
        self.progress
            .fetch_add(amount, std::sync::atomic::Ordering::Acquire);
        // Offset the bytes at last offset by this amount
        self.progress_object
            .bytes_last_update
            .fetch_add(amount, Ordering::Acquire);
        // Dont' fire update
    }
}

impl ProgressObject {
    pub fn new(max: usize, length: usize, sender: Sender<DownloadManagerSignal>) -> Self {
        let arr = Mutex::new((0..length).map(|_| Arc::new(AtomicUsize::new(0))).collect());
        Self {
            max: Arc::new(Mutex::new(max)),
            progress_instances: Arc::new(arr),
            start: Arc::new(Mutex::new(Instant::now())),
            sender,

            last_update_time: Arc::new(AtomicInstant::now()),
            bytes_last_update: Arc::new(AtomicUsize::new(0)),
            rolling: RollingProgressWindow::new(),
        }
    }

    pub fn set_time_now(&self) {
        *lock!(self.start) = Instant::now();
    }
    pub fn sum(&self) -> usize {
        lock!(self.progress_instances)
            .iter()
            .map(|instance| instance.load(Ordering::Acquire))
            .sum()
    }
    pub fn reset(&self) {
        self.set_time_now();
        self.bytes_last_update.store(0, Ordering::Release);
        self.rolling.reset();
        lock!(self.progress_instances)
            .iter()
            .for_each(|x| x.store(0, Ordering::SeqCst));
    }
    pub fn get_max(&self) -> usize {
        *lock!(self.max)
    }
    pub fn set_max(&self, new_max: usize) {
        *lock!(self.max) = new_max;
    }
    pub fn set_size(&self, length: usize) {
        *lock!(self.progress_instances) =
            (0..length).map(|_| Arc::new(AtomicUsize::new(0))).collect();
    }
    pub fn get_progress(&self) -> f64 {
        self.sum() as f64 / self.get_max() as f64
    }
    pub fn get(&self, index: usize) -> Arc<AtomicUsize> {
        lock!(self.progress_instances)[index].clone()
    }
    fn update_window(&self, kilobytes_per_second: usize) {
        self.rolling.update(kilobytes_per_second);
    }
}

#[throttle(1, Duration::from_millis(20))]
pub fn calculate_update(progress: &ProgressObject) {
    let last_update_time = progress
        .last_update_time
        .swap(Instant::now(), Ordering::SeqCst);
    let time_since_last_update = Instant::now()
        .duration_since(last_update_time)
        .as_millis_f64();

    let current_bytes_downloaded = progress.sum();
    let max = progress.get_max();
    let bytes_at_last_update = progress
        .bytes_last_update
        .swap(current_bytes_downloaded, Ordering::Acquire);

    let bytes_since_last_update =
        current_bytes_downloaded.saturating_sub(bytes_at_last_update) as f64;

    let kilobytes_per_second = bytes_since_last_update / time_since_last_update;

    let bytes_remaining = max.saturating_sub(current_bytes_downloaded); // bytes

    progress.update_window(kilobytes_per_second as usize);
    push_update(progress, bytes_remaining);
}

#[throttle(1, Duration::from_millis(250))]
pub fn push_update(progress: &ProgressObject, bytes_remaining: usize) {
    let average_speed = progress.rolling.get_average();
    let time_remaining = (bytes_remaining / 1000) / average_speed.max(1);

    update_ui(progress, average_speed, time_remaining);
    update_queue(progress);
}

fn update_ui(progress_object: &ProgressObject, kilobytes_per_second: usize, time_remaining: usize) {
    send!(
        progress_object.sender,
        DownloadManagerSignal::UpdateUIStats(kilobytes_per_second, time_remaining)
    );
}

fn update_queue(progress: &ProgressObject) {
    send!(progress.sender, DownloadManagerSignal::UpdateUIQueue)
}
