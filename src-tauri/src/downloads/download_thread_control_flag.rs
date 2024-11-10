use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub enum DownloadThreadControlFlag {
    Stop,
    Go,
}
impl From<DownloadThreadControlFlag> for bool {
    fn from(value: DownloadThreadControlFlag) -> Self {
        match value {
            DownloadThreadControlFlag::Stop => false,
            DownloadThreadControlFlag::Go => true,
        }
    }
}


#[derive(Clone)]
pub struct DownloadThreadControl {
    inner: Arc<AtomicBool>,
}

impl DownloadThreadControl {
    pub fn new(flag: DownloadThreadControlFlag) -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(flag.into())),
        }
    }
    pub fn get(&self) -> bool {
        self.inner.load(Ordering::Relaxed)
    }
    pub fn set(&self, flag: DownloadThreadControlFlag) {
        self.inner.store(flag.into(), Ordering::Relaxed);
    }
}
