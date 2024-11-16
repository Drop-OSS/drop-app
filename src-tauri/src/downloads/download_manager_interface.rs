use std::{
    any::Any, collections::VecDeque, sync::{
        mpsc::{SendError, Sender},
        Arc, Mutex, MutexGuard,
    }, thread::JoinHandle
};

use log::info;

use super::{download_agent::GameDownloadAgent, download_manager::{DownloadManagerSignal, DownloadManagerStatus, GameDownloadStatus}, progress_object::ProgressObject};

/// Accessible front-end for the DownloadManager
/// 
/// The system works entirely through signals, both internally and externally,
/// all of which are accessible through the DownloadManagerSignal type, but 
/// should not be used directly. Rather, signals are abstracted through this
/// interface.
/// 
/// The actual download queue may be accessed through the .edit() function,
/// which provides raw access to the underlying queue. 
/// THIS EDITING IS BLOCKING!!!
pub struct DownloadManagerInterface {
    terminator: JoinHandle<Result<(), ()>>,
    download_queue: Arc<Mutex<VecDeque<Arc<AgentInterfaceData>>>>,
    progress: Arc<Mutex<Option<ProgressObject>>>,
    command_sender: Sender<DownloadManagerSignal>,
}
pub struct AgentInterfaceData {
    pub id: String,
    pub status: Mutex<GameDownloadStatus>,
}
impl From<Arc<GameDownloadAgent>> for AgentInterfaceData {
    fn from(value: Arc<GameDownloadAgent>) -> Self {
        Self {
            id: value.id.clone(),
            status: Mutex::from(GameDownloadStatus::Uninitialised)
        }
    }
}

impl DownloadManagerInterface {
    pub fn new(
        terminator: JoinHandle<Result<(), ()>>,
        download_queue: Arc<Mutex<VecDeque<Arc<AgentInterfaceData>>>>,
        progress: Arc<Mutex<Option<ProgressObject>>>,
        command_sender: Sender<DownloadManagerSignal>,
    ) -> Self {
        Self {
            terminator,
            download_queue,
            progress,
            command_sender,
        }
    }

    pub fn queue_game(
        &self,
        id: String,
        version: String,
        target_download_dir: usize,
    ) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("Adding game id {}", id);
        self.command_sender.send(DownloadManagerSignal::Queue(
            id,
            version,
            target_download_dir,
        ))?;
        self.command_sender.send(DownloadManagerSignal::Go)
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<Arc<AgentInterfaceData>>> {
        self.download_queue.lock().unwrap()
    }
    pub fn read_queue(&self) -> VecDeque<Arc<AgentInterfaceData>> {
        self.download_queue.lock().unwrap().clone()
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
    pub fn remove_from_queue_string(&self, id: String) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, id).unwrap();
        queue.remove(current_index);
    }
    pub fn pause_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.command_sender.send(DownloadManagerSignal::Stop)
    }
    pub fn resume_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.command_sender.send(DownloadManagerSignal::Go)
    }
    pub fn ensure_terminated(self) -> Result<Result<(),()>, Box<dyn Any + Send>> {
        self.command_sender.send(DownloadManagerSignal::Finish).unwrap();
        self.terminator.join()
    }
}

/// Takes in the locked value from .edit() and attempts to
/// get the index of whatever game_id is passed in
fn get_index_from_id(queue: &mut MutexGuard<'_, VecDeque<Arc<AgentInterfaceData>>>, id: String) -> Option<usize> {
    queue
        .iter()
        .position(|download_agent| download_agent.id == id)
}
