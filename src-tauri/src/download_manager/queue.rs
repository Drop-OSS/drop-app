use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use super::downloadable_metadata::DownloadableMetadata;

#[derive(Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<DownloadableMetadata>>>,
}

#[allow(dead_code)]
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
    pub fn empty(&self) -> bool {
        self.inner.lock().unwrap().len() == 0
    }
    pub fn exists(&self, meta: DownloadableMetadata) -> bool {
        self.read().contains(&meta)
    }
    /// Either inserts `interface` at the specified index, or appends to
    /// the back of the deque if index is greater than the length of the deque
    pub fn insert(&self, interface: DownloadableMetadata, index: usize) {
        if self.read().len() > index {
            self.append(interface);
        } else {
            self.edit().insert(index, interface);
        }
    }
    pub fn append(&self, interface: DownloadableMetadata) {
        self.edit().push_back(interface);
    }
    pub fn pop_front_if_equal(
        &self,
        meta: &DownloadableMetadata,
    ) -> Option<DownloadableMetadata> {
        let mut queue = self.edit();
        let front = match queue.front() {
            Some(front) => front,
            None => return None,
        };
        if front == meta {
            return queue.pop_front();
        }
        None
    }
    pub fn get_by_meta(&self, meta: &DownloadableMetadata) -> Option<usize> {
        self.read().iter().position(|data| data == meta)
    }
    pub fn move_to_index_by_meta(&self, meta: &DownloadableMetadata, new_index: usize) -> Result<(), ()> {
        let index = match self.get_by_meta(meta) {
            Some(index) => index,
            None => return Err(()),
        };
        let existing = match self.edit().remove(index) {
            Some(existing) => existing,
            None => return Err(()),
        };
        self.edit().insert(new_index, existing);
        Ok(())
    }
}
