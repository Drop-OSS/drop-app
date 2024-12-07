use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use super::download_manager::GameDownloadAgentQueueStandin;

#[derive(Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<Arc<GameDownloadAgentQueueStandin>>>>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn read(&self) -> VecDeque<Arc<GameDownloadAgentQueueStandin>> {
        self.inner.lock().unwrap().clone()
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<Arc<GameDownloadAgentQueueStandin>>> {
        self.inner.lock().unwrap()
    }
    pub fn pop_front(&self) -> Option<Arc<GameDownloadAgentQueueStandin>> {
        self.edit().pop_front()
    }
    pub fn empty(&self) -> bool {
        self.inner.lock().unwrap().len() == 0
    }
    /// Either inserts `interface` at the specified index, or appends to
    /// the back of the deque if index is greater than the length of the deque
    pub fn insert(&self, interface: GameDownloadAgentQueueStandin, index: usize) {
        if self.read().len() > index {
            self.append(interface);
        } else {
            self.edit().insert(index, Arc::new(interface));
        }
    }
    pub fn append(&self, interface: GameDownloadAgentQueueStandin) {
        self.edit().push_back(Arc::new(interface));
    }
    pub fn pop_front_if_equal(&self, game_id: String) -> Option<Arc<GameDownloadAgentQueueStandin>> {
        let mut queue = self.edit();
        let front = match queue.front() {
            Some(front) => front,
            None => return None,
        };
        if front.id == game_id {
            return queue.pop_front();
        }
        return None;
    }
    pub fn get_by_id(&self, game_id: String) -> Option<usize> {
        self.read().iter().position(|data| data.id == game_id)
    }
    pub fn move_to_index_by_id(&self, game_id: String, new_index: usize) -> Result<(), ()> {
        let index = match self.get_by_id(game_id) {
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
