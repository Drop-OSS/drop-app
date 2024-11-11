use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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
impl From<bool> for DownloadThreadControlFlag {
    fn from(value: bool) -> Self {
        match value {
            true => DownloadThreadControlFlag::Go,
            false => DownloadThreadControlFlag::Stop,
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
    pub fn get(&self) -> DownloadThreadControlFlag {
        self.inner.load(Ordering::Relaxed).into()
    }
    pub fn set(&self, flag: DownloadThreadControlFlag) {
        self.inner.store(flag.into(), Ordering::Relaxed);
    }
}
