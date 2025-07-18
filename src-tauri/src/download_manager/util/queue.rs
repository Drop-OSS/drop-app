use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::database::models::data::DownloadableMetadata;

#[derive(Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<DownloadableMetadata>>>,
}

#[allow(dead_code)]
impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn read(&self) -> VecDeque<DownloadableMetadata> {
        self.inner.lock().unwrap().clone()
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<DownloadableMetadata>> {
        self.inner.lock().unwrap()
    }
    pub fn pop_front(&self) -> Option<DownloadableMetadata> {
        self.edit().pop_front()
    }
    pub fn exists(&self, meta: DownloadableMetadata) -> bool {
        self.read().contains(&meta)
    }
    pub fn append(&self, interface: DownloadableMetadata) {
        self.edit().push_back(interface);
    }
    pub fn get_by_meta(&self, meta: &DownloadableMetadata) -> Option<usize> {
        self.read().iter().position(|data| data == meta)
    }
}
