use std::{
    any::Any,
    collections::VecDeque,
    fmt::Debug,
    sync::{
        mpsc::{SendError, Sender},
        Mutex, MutexGuard,
    },
    thread::JoinHandle,
};

use log::{debug, info};
use serde::Serialize;

use crate::error::application_download_error::ApplicationDownloadError;

use super::{
    download_manager_builder::{CurrentProgressObject, DownloadAgent},
    downloadable_metadata::DownloadableMetadata,
    queue::Queue,
};

pub enum DownloadManagerSignal {
    /// Resumes (or starts) the DownloadManager
    Go,
    /// Pauses the DownloadManager
    Stop,
    /// Called when a DownloadAgent has fully completed a download.
    Completed(DownloadableMetadata),
    /// Generates and appends a DownloadAgent
    /// to the registry and queue
    Queue(DownloadAgent),
    /// Tells the Manager to stop the current
    /// download, sync everything to disk, and
    /// then exit
    Finish,
    /// Stops, removes, and tells a download to cleanup
    Cancel(DownloadableMetadata),
    /// Removes a given application
    Remove(DownloadableMetadata),
    /// Any error which occurs in the agent
    Error(ApplicationDownloadError),
    /// Pushes UI update
    UpdateUIQueue,
    UpdateUIStats(usize, usize), //kb/s and seconds
    /// Uninstall download
    /// Takes download ID
    Uninstall(DownloadableMetadata),
}

#[derive(Debug)]
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

#[derive(Serialize, Clone, Debug)]
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
    terminator: Mutex<Option<JoinHandle<Result<(), ()>>>>,
    download_queue: Queue,
    progress: CurrentProgressObject,
    command_sender: Sender<DownloadManagerSignal>,
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
            terminator: Mutex::new(Some(terminator)),
            download_queue,
            progress,
            command_sender,
        }
    }

    pub fn queue_download(
        &self,
        download: DownloadAgent,
    ) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("creating download with meta {:?}", download.metadata());
        self.command_sender
            .send(DownloadManagerSignal::Queue(download))?;
        self.command_sender.send(DownloadManagerSignal::Go)
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<DownloadableMetadata>> {
        self.download_queue.edit()
    }
    pub fn read_queue(&self) -> VecDeque<DownloadableMetadata> {
        self.download_queue.read()
    }
    pub fn get_current_download_progress(&self) -> Option<f64> {
        let progress_object = (*self.progress.lock().unwrap()).clone()?;
        Some(progress_object.get_progress())
    }
    pub fn rearrange_string(&self, meta: &DownloadableMetadata, new_index: usize) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, meta).unwrap();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
        self.command_sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }
    pub fn cancel(&self, meta: DownloadableMetadata) {
        self.command_sender
            .send(DownloadManagerSignal::Cancel(meta))
            .unwrap();
    }
    pub fn rearrange(&self, current_index: usize, new_index: usize) {
        if current_index == new_index {
            return;
        };

        let needs_pause = current_index == 0 || new_index == 0;
        if needs_pause {
            self.command_sender
                .send(DownloadManagerSignal::Stop)
                .unwrap();
        }

        debug!(
            "moving download at index {} to index {}",
            current_index, new_index
        );

        let mut queue = self.edit();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
        drop(queue);

        if needs_pause {
            self.command_sender.send(DownloadManagerSignal::Go).unwrap();
        }
        self.command_sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
        self.command_sender
            .send(DownloadManagerSignal::Go)
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
    pub fn ensure_terminated(&self) -> Result<Result<(), ()>, Box<dyn Any + Send>> {
        self.command_sender
            .send(DownloadManagerSignal::Finish)
            .unwrap();
        let terminator = self.terminator.lock().unwrap().take();
        terminator.unwrap().join()
    }
    pub fn uninstall_application(&self, meta: DownloadableMetadata) {
        self.command_sender
            .send(DownloadManagerSignal::Uninstall(meta))
            .unwrap();
    }
    pub fn get_sender(&self) -> Sender<DownloadManagerSignal> {
        self.command_sender.clone()
    }
}

/// Takes in the locked value from .edit() and attempts to
/// get the index of whatever id is passed in
fn get_index_from_id(
    queue: &mut MutexGuard<'_, VecDeque<DownloadableMetadata>>,
    meta: &DownloadableMetadata,
) -> Option<usize> {
    queue
        .iter()
        .position(|download_agent| download_agent == meta)
}
