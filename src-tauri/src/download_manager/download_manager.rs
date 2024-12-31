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

use crate::downloads::download_agent::GameDownloadAgent;

use super::{application_download_error::ApplicationDownloadError, download_manager_builder::{CurrentProgressObject, DownloadableQueueStandin}, downloadable::Downloadable, queue::Queue};

pub enum DownloadType {
    Game,
    Tool,
}
impl DownloadType {
    pub fn generate(
        &self,
        id: String,
        version: String,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>) -> Box<dyn Downloadable + Send + Sync> {
        return Box::new(match self {
            DownloadType::Game => GameDownloadAgent::new(
                id.clone(),
                version,
                target_download_dir,
                sender.clone(),
            ),
            DownloadType::Tool => todo!(),
        })
    }
}

pub enum DownloadManagerSignal {
    /// Resumes (or starts) the DownloadManager
    Go,
    /// Pauses the DownloadManager
    Stop,
    /// Called when a DownloadAgent has fully completed a download.
    Completed(String),
    /// Generates and appends a DownloadAgent
    /// to the registry and queue
    Queue(String, String, usize),
    /// Tells the Manager to stop the current
    /// download, sync everything to disk, and
    /// then exit
    Finish,
    /// Stops (but doesn't remove) current download
    Cancel,
    /// Removes a given application
    Remove(String),
    /// Any error which occurs in the agent
    Error(ApplicationDownloadError),
    /// Pushes UI update
    UpdateUIQueue,
    UpdateUIStats(usize, usize), //kb/s and seconds
    /// Uninstall download
    /// Takes download ID
    Uninstall(String),
}

#[derive(Debug, Clone)]
pub enum DownloadManagerStatus {
    Downloading,
    Paused,
    Empty,
    Error(ApplicationDownloadError),
    Finished,
}

impl Serialize for DownloadManagerStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!["{:?}", self])
    }
}

#[derive(Serialize, Clone)]
pub enum DownloadStatus {
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
impl<T: Downloadable> From<Arc<T>> for DownloadableQueueStandin {
    fn from(value: Arc<T>) -> Self {
        Self {
            id: value.id(),
            status: Mutex::from(DownloadStatus::Queued),
            progress: value.progress().clone(),
        }
    }
}
impl Debug for DownloadableQueueStandin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadableQueueStandin")
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

    pub fn queue_download(
        &self,
        id: String,
        version: String,
        target_download_dir: usize,
    ) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("Adding download id {}", id);
        self.command_sender.send(DownloadManagerSignal::Queue(
            id,
            version,
            target_download_dir,
        ))?;
        self.command_sender.send(DownloadManagerSignal::Go)
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<Arc<DownloadableQueueStandin>>> {
        self.download_queue.edit()
    }
    pub fn read_queue(&self) -> VecDeque<Arc<DownloadableQueueStandin>> {
        self.download_queue.read()
    }
    pub fn get_current_download_progress(&self) -> Option<f64> {
        let progress_object = (*self.progress.lock().unwrap()).clone()?;
        Some(progress_object.get_progress())
    }
    pub fn rearrange_string(&self, id: String, new_index: usize) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, id).unwrap();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
        self.command_sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }
    pub fn cancel(&self, id: String) {
        self.command_sender
            .send(DownloadManagerSignal::Remove(id))
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
        drop(queue);

        if needs_pause {
            self.command_sender.send(DownloadManagerSignal::Go).unwrap();
        }
        self.command_sender
            .send(DownloadManagerSignal::UpdateUIQueue)
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
    pub fn uninstall_application(&self, id: String) {
        self.command_sender
            .send(DownloadManagerSignal::Uninstall(id))
            .unwrap();
    }
}

/// Takes in the locked value from .edit() and attempts to
/// get the index of whatever id is passed in
fn get_index_from_id(
    queue: &mut MutexGuard<'_, VecDeque<Arc<DownloadableQueueStandin>>>,
    id: String,
) -> Option<usize> {
    queue
        .iter()
        .position(|download_agent| download_agent.id == id)
}
