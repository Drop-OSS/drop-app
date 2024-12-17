use std::{
    any::Any,
    collections::VecDeque,
    fmt::Debug,
    sync::{
        mpsc::{SendError, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::JoinHandle,
};

use log::info;
use serde::Serialize;

use super::{
    download_agent::{GameDownloadAgent, GameDownloadError},
    download_manager_builder::CurrentProgressObject,
    progress_object::ProgressObject,
    queue::Queue,
};

pub enum DownloadManagerSignal {
    /// Resumes (or starts) the DownloadManager
    Go,
    /// Pauses the DownloadManager
    Stop,
    /// Called when a GameDownloadAgent has fully completed a download.
    Completed(String),
    /// Generates and appends a GameDownloadAgent
    /// to the registry and queue
    Queue(String, String, usize),
    /// Tells the Manager to stop the current
    /// download and return
    Finish,
    Cancel,
    /// Any error which occurs in the agent
    Error(GameDownloadError),
    /// Pushes UI update
    Update,
    /// Causes the Download Agent status to be synced to disk
    Sync(usize),
}
pub enum DownloadManagerStatus {
    Downloading,
    Paused,
    Empty,
    Error(GameDownloadError),
}

#[derive(Serialize, Clone)]
pub enum GameDownloadStatus {
    Queued,
    Downloading,
    Error,
}

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
pub struct DownloadManager {
    terminator: JoinHandle<Result<(), ()>>,
    download_queue: Queue,
    progress: CurrentProgressObject,
    command_sender: Sender<DownloadManagerSignal>,
}
pub struct GameDownloadAgentQueueStandin {
    pub id: String,
    pub status: Mutex<GameDownloadStatus>,
    pub progress: Arc<ProgressObject>,
}
impl From<Arc<GameDownloadAgent>> for GameDownloadAgentQueueStandin {
    fn from(value: Arc<GameDownloadAgent>) -> Self {
        Self {
            id: value.id.clone(),
            status: Mutex::from(GameDownloadStatus::Queued),
            progress: value.progress.clone(),
        }
    }
}
impl Debug for GameDownloadAgentQueueStandin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameDownloadAgentQueueStandin")
            .field("id", &self.id)
            .finish()
    }
}

#[allow(dead_code)]
impl DownloadManager {
    pub fn new(
        terminator: JoinHandle<Result<(), ()>>,
        download_queue: Queue,
        progress: CurrentProgressObject,
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
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<Arc<GameDownloadAgentQueueStandin>>> {
        self.download_queue.edit()
    }
    pub fn read_queue(&self) -> VecDeque<Arc<GameDownloadAgentQueueStandin>> {
        self.download_queue.read()
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
        self.command_sender
            .send(DownloadManagerSignal::Update)
            .unwrap();
    }
    pub fn rearrange(&self, current_index: usize, new_index: usize) {
        if current_index == new_index {
            return;
        };

        let needs_pause = current_index == 0 || new_index == 0;
        if needs_pause {
            self.command_sender
                .send(DownloadManagerSignal::Cancel)
                .unwrap();
        }

        info!("moving {} to {}", current_index, new_index);

        let mut queue = self.edit();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);

        info!("new queue: {:?}", queue);

        if needs_pause {
            self.command_sender.send(DownloadManagerSignal::Go).unwrap();
        }
        self.command_sender
            .send(DownloadManagerSignal::Update)
            .unwrap();
    }
    pub fn pause_downloads(&self) {
        self.command_sender
            .send(DownloadManagerSignal::Stop)
            .unwrap();
    }
    pub fn resume_downloads(&self) {
        self.command_sender.send(DownloadManagerSignal::Go).unwrap();
    }
    pub fn ensure_terminated(self) -> Result<Result<(), ()>, Box<dyn Any + Send>> {
        self.command_sender
            .send(DownloadManagerSignal::Finish)
            .unwrap();
        self.terminator.join()
    }
}

/// Takes in the locked value from .edit() and attempts to
/// get the index of whatever game_id is passed in
fn get_index_from_id(
    queue: &mut MutexGuard<'_, VecDeque<Arc<GameDownloadAgentQueueStandin>>>,
    id: String,
) -> Option<usize> {
    queue
        .iter()
        .position(|download_agent| download_agent.id == id)
}
