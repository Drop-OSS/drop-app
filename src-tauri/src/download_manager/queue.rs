use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use super::download_manager_builder::DownloadableQueueStandin;

#[derive(Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<Arc<DownloadableQueueStandin>>>>,
}

#[allow(dead_code)]
impl Queue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn read(&self) -> VecDeque<Arc<DownloadableQueueStandin>> {
        self.inner.lock().unwrap().clone()
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<Arc<DownloadableQueueStandin>>> {
        self.inner.lock().unwrap()
    }
    pub fn pop_front(&self) -> Option<Arc<DownloadableQueueStandin>> {
        self.edit().pop_front()
    }
    pub fn empty(&self) -> bool {
        self.inner.lock().unwrap().len() == 0
    }
    /// Either inserts `interface` at the specified index, or appends to
    /// the back of the deque if index is greater than the length of the deque
    pub fn insert(&self, interface: DownloadableQueueStandin, index: usize) {
        if self.read().len() > index {
            self.append(interface);
        } else {
            self.edit().insert(index, Arc::new(interface));
        }
    }
    pub fn append(&self, interface: DownloadableQueueStandin) {
        self.edit().push_back(Arc::new(interface));
    }
    pub fn pop_front_if_equal(
        &self,
        id: String,
    ) -> Option<Arc<DownloadableQueueStandin>> {
        let mut queue = self.edit();
        let front = match queue.front() {
            Some(front) => front,
            None => return None,
        };
        if front.id == id {
            return queue.pop_front();
        }
        None
    }
    pub fn get_by_id(&self, id: String) -> Option<usize> {
        self.read().iter().position(|data| data.id == id)
    }
    pub fn move_to_index_by_id(&self, id: String, new_index: usize) -> Result<(), ()> {
        let index = match self.get_by_id(id) {
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
