use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex, MutexGuard},
};

use database::DownloadableMetadata;
use log::{debug, info};
use serde::Serialize;
use tauri::async_runtime::JoinHandle;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::SendError;
use utils::{lock, send};

use crate::{depot_manager::DepotManager, error::ApplicationDownloadError};

use super::{
    download_manager_builder::{CurrentProgressObject, DownloadAgent},
    util::queue::Queue,
};

#[derive(Debug)]
pub enum DownloadManagerSignal {
    /// Resumes (or starts) the `DownloadManager`
    Go,
    /// Pauses the `DownloadManager`
    Stop,
    /// Called when a `DownloadAgent` has fully completed a download.
    Completed(DownloadableMetadata),
    /// Generates and appends a `DownloadAgent`
    /// to the registry and queue
    Queue(DownloadAgent),
    /// Tells the Manager to stop the current
    /// download, sync everything to disk, and
    /// then exit
    Finish,
    /// Stops, removes, and tells a download to cleanup
    Cancel(DownloadableMetadata),
    /// Any error which occurs in the agent
    Error(ApplicationDownloadError),
    /// Pushes UI update
    UpdateUIQueue,
    UpdateUIStats(usize, usize), //kb/s and seconds
}

#[derive(Debug)]
pub enum DownloadManagerStatus {
    Downloading,
    Paused,
    Empty,
    Error,
}

impl Serialize for DownloadManagerStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!["{self:?}"])
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Validating,
    Error,
}

/// Accessible front-end for the `DownloadManager`
///
/// The system works entirely through signals, both internally and externally,
/// all of which are accessible through the `DownloadManagerSignal` type, but
/// should not be used directly. Rather, signals are abstracted through this
/// interface.
///
/// The actual download queue may be accessed through the .`edit()` function,
/// which provides raw access to the underlying queue.
/// THIS EDITING IS BLOCKING!!!
pub struct DownloadManager {
    terminator: Mutex<Option<JoinHandle<()>>>,
    download_queue: Queue,
    progress: CurrentProgressObject,
    command_sender: Sender<DownloadManagerSignal>,
    depot_manager: Arc<DepotManager>,
}

impl Debug for DownloadManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadManager").finish()
    }
}

#[allow(dead_code)]
impl DownloadManager {
    pub fn new(
        terminator: JoinHandle<()>,
        download_queue: Queue,
        progress: CurrentProgressObject,
        command_sender: Sender<DownloadManagerSignal>,
        depot_manager: Arc<DepotManager>,
    ) -> Self {
        Self {
            terminator: Mutex::new(Some(terminator)),
            download_queue,
            progress,
            command_sender,
            depot_manager
        }
    }

    pub async fn queue_download(
        &self,
        download: DownloadAgent,
    ) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("creating download with meta {:?}", download.metadata());
        self.command_sender
            .send(DownloadManagerSignal::Queue(download))
            .await?;
        self.command_sender.send(DownloadManagerSignal::Go).await
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<DownloadableMetadata>> {
        self.download_queue.edit()
    }
    pub fn read_queue(&self) -> VecDeque<DownloadableMetadata> {
        self.download_queue.read()
    }
    pub fn get_current_download_progress(&self) -> Option<f64> {
        let progress_object = (*lock!(self.progress)).clone()?;
        Some(progress_object.get_progress())
    }
    pub async fn rearrange_string(&self, meta: &DownloadableMetadata, new_index: usize) {
        let mut queue = self.edit();
        let current_index =
            get_index_from_id(&mut queue, meta).expect("Failed to get meta index from id");
        let to_move = queue
            .remove(current_index)
            .expect("Failed to remove meta at index from queue");
        queue.insert(new_index, to_move);
        send!(self.command_sender, DownloadManagerSignal::UpdateUIQueue);
    }
    pub async fn cancel(&self, meta: DownloadableMetadata) {
        send!(self.command_sender, DownloadManagerSignal::Cancel(meta));
    }
    pub async fn rearrange(&self, current_index: usize, new_index: usize) {
        if current_index == new_index {
            return;
        }

        let needs_pause = current_index == 0 || new_index == 0;
        if needs_pause {
            send!(self.command_sender, DownloadManagerSignal::Stop);
        }

        debug!("moving download at index {current_index} to index {new_index}");

        {
            let mut queue = self.edit();
            let to_move = queue.remove(current_index).expect("Failed to get");
            queue.insert(new_index, to_move);
        }

        if needs_pause {
            send!(self.command_sender, DownloadManagerSignal::Go);
        }
        send!(self.command_sender, DownloadManagerSignal::UpdateUIQueue);
        send!(self.command_sender, DownloadManagerSignal::Go);
    }
    pub async fn pause_downloads(&self) {
        send!(self.command_sender, DownloadManagerSignal::Stop);
    }
    pub async fn resume_downloads(&self) {
        send!(self.command_sender, DownloadManagerSignal::Go);
    }
    pub async fn ensure_terminated(&self) -> Result<(), tauri::Error> {
        send!(self.command_sender, DownloadManagerSignal::Finish);
        let terminator = lock!(self.terminator).take();
        terminator.unwrap().await
    }
    pub fn get_sender(&self) -> Sender<DownloadManagerSignal> {
        self.command_sender.clone()
    }
    pub fn clone_depot_manager(&self) -> Arc<DepotManager> {
        self.depot_manager.clone()
    }
}

/// Takes in the locked value from .`edit()` and attempts to
/// get the index of whatever id is passed in
fn get_index_from_id(
    queue: &mut MutexGuard<'_, VecDeque<DownloadableMetadata>>,
    meta: &DownloadableMetadata,
) -> Option<usize> {
    queue
        .iter()
        .position(|download_agent| download_agent == meta)
}
