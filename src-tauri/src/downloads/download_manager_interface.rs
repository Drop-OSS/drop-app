use std::{collections::VecDeque, sync::{mpsc::{SendError, Sender}, Arc, Mutex, MutexGuard}, thread::JoinHandle};

use log::info;

use super::{download_manager::DownloadManagerSignal, progress_object::ProgressObject};

pub struct DownloadManagerInterface {
    terminator: JoinHandle<Result<(),()>>,
    download_queue: Arc<Mutex<VecDeque<String>>>,
    progress: Arc<Mutex<Option<ProgressObject>>>,
    sender: Sender<DownloadManagerSignal>,
}

impl DownloadManagerInterface {
    
    pub fn new(
        terminator: JoinHandle<Result<(),()>>,
         download_queue: Arc<Mutex<VecDeque<String>>>,
          progress: Arc<Mutex<Option<ProgressObject>>>,
          sender: Sender<DownloadManagerSignal>) -> Self {
        Self { terminator, download_queue, progress, sender }
    }
    
    pub fn queue_game(&self, game_id: String, version: String, target_download_dir: usize) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("Adding game id {}", game_id);
        self.sender.send(DownloadManagerSignal::Queue(game_id, version, target_download_dir))?;
        self.sender.send(DownloadManagerSignal::Go)
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<String>> {
        self.download_queue.lock().unwrap()
    }
    pub fn get_current_game_download_progress(&self) -> Option<f64> {
        let progress_object = (*self.progress.lock().unwrap()).clone()?;
        Some(progress_object.get_progress())
    }
    pub fn rearrange_string(&self, id: String, new_index: usize) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, id).unwrap();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
    }
    pub fn rearrange(&self, current_index: usize, new_index: usize) {
        let mut queue = self.edit();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
    }
    pub fn remove_from_queue(&self, index: usize) {
        self.edit().remove(index);
    }
    pub fn remove_from_queue_string(&self, game_id: String) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, game_id).unwrap();
        queue.remove(current_index);
    }
    pub fn pause_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.sender.send(DownloadManagerSignal::Stop)
    }
    pub fn resume_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.sender.send(DownloadManagerSignal::Go)
    }
    pub fn ensure_terminated(self) -> Result<(), ()> {
        match self.terminator.join() {
            Ok(o) => o,
            Err(_) => Err(()),
        }
    }
}

pub fn get_index_from_id(queue: &mut MutexGuard<'_, VecDeque<String>>, id: String) -> Option<usize> {
    queue.iter().position(|download_agent| {
        download_agent == &id
    })
}
