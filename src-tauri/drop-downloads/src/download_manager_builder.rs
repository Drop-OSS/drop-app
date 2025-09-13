use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{JoinHandle, spawn},
};

use drop_database::models::data::DownloadableMetadata;
use drop_errors::application_download_error::ApplicationDownloadError;
use log::{debug, error, info, warn};
use tauri::{AppHandle, Emitter};

use crate::{
    download_manager_frontend::DownloadStatus, events::{QueueUpdateEvent, QueueUpdateEventQueueData, StatsUpdateEvent}
};

use super::{
    download_manager_frontend::{DownloadManager, DownloadManagerSignal, DownloadManagerStatus},
    downloadable::Downloadable,
    util::{
        download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
        progress_object::ProgressObject,
        queue::Queue,
    },
};

pub type DownloadAgent = Arc<Box<dyn Downloadable + Send + Sync>>;
pub type CurrentProgressObject = Arc<Mutex<Option<Arc<ProgressObject>>>>;

pub struct DownloadManagerBuilder {
    download_agent_registry: HashMap<DownloadableMetadata, DownloadAgent>,
    download_queue: Queue,
    command_receiver: Receiver<DownloadManagerSignal>,
    sender: Sender<DownloadManagerSignal>,
    progress: CurrentProgressObject,
    status: Arc<Mutex<DownloadManagerStatus>>,
    app_handle: AppHandle,

    current_download_thread: Mutex<Option<JoinHandle<()>>>,
    active_control_flag: Option<DownloadThreadControl>,
}
impl DownloadManagerBuilder {
    pub fn build(app_handle: AppHandle) -> DownloadManager {
        let queue = Queue::new();
        let (command_sender, command_receiver) = channel();
        let active_progress = Arc::new(Mutex::new(None));
        let status = Arc::new(Mutex::new(DownloadManagerStatus::Empty));

        let manager = Self {
            download_agent_registry: HashMap::new(),
            download_queue: queue.clone(),
            command_receiver,
            status: status.clone(),
            sender: command_sender.clone(),
            progress: active_progress.clone(),
            app_handle,

            current_download_thread: Mutex::new(None),
            active_control_flag: None,
        };

        let terminator = spawn(|| manager.manage_queue());

        DownloadManager::new(terminator, queue, active_progress, command_sender)
    }

    fn set_status(&self, status: DownloadManagerStatus) {
        *self.status.lock().unwrap() = status;
    }

    fn remove_and_cleanup_front_download(&mut self, meta: &DownloadableMetadata) -> DownloadAgent {
        self.download_queue.pop_front();
        let download_agent = self.download_agent_registry.remove(meta).unwrap();
        self.cleanup_current_download();
        download_agent
    }

    // CAREFUL WITH THIS FUNCTION
    // Make sure the download thread is terminated
    fn cleanup_current_download(&mut self) {
        self.active_control_flag = None;
        *self.progress.lock().unwrap() = None;

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();

        if let Some(unfinished_thread) = download_thread_lock.take()
            && !unfinished_thread.is_finished()
        {
            unfinished_thread.join().unwrap();
        }
        drop(download_thread_lock);
    }

    fn stop_and_wait_current_download(&self) -> bool {
        self.set_status(DownloadManagerStatus::Paused);
        if let Some(current_flag) = &self.active_control_flag {
            current_flag.set(DownloadThreadControlFlag::Stop);
        }

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        if let Some(current_download_thread) = download_thread_lock.take() {
            return current_download_thread.join().is_ok();
        };

        true
    }

    fn manage_queue(mut self) -> Result<(), ()> {
        loop {
            let signal = match self.command_receiver.recv() {
                Ok(signal) => signal,
                Err(_) => return Err(()),
            };

            match signal {
                DownloadManagerSignal::Go => {
                    self.manage_go_signal();
                }
                DownloadManagerSignal::Stop => {
                    self.manage_stop_signal();
                }
                DownloadManagerSignal::Completed(meta) => {
                    self.manage_completed_signal(meta);
                }
                DownloadManagerSignal::Queue(download_agent) => {
                    self.manage_queue_signal(download_agent);
                }
                DownloadManagerSignal::Error(e) => {
                    self.manage_error_signal(e);
                }
                DownloadManagerSignal::UpdateUIQueue => {
                    self.push_ui_queue_update();
                }
                DownloadManagerSignal::UpdateUIStats(kbs, time) => {
                    self.push_ui_stats_update(kbs, time);
                }
                DownloadManagerSignal::Finish => {
                    self.stop_and_wait_current_download();
                    return Ok(());
                }
                DownloadManagerSignal::Cancel(meta) => {
                    self.manage_cancel_signal(&meta);
                }
            }
        }
    }
    fn manage_queue_signal(&mut self, download_agent: DownloadAgent) {
        debug!("got signal Queue");
        let meta = download_agent.metadata();

        debug!("queue metadata: {meta:?}");

        if self.download_queue.exists(meta.clone()) {
            warn!("download with same ID already exists");
            return;
        }

        download_agent.on_queued(&self.app_handle);
        self.download_queue.append(meta.clone());
        self.download_agent_registry.insert(meta, download_agent);

        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }

    fn manage_go_signal(&mut self) {
        debug!("got signal Go");
        if self.download_agent_registry.is_empty() {
            debug!(
                "Download agent registry: {:?}",
                self.download_agent_registry.len()
            );
            return;
        }

        debug!("current download queue: {:?}", self.download_queue.read());

        let agent_data = if let Some(agent_data) = self.download_queue.read().front() {
            agent_data.clone()
        } else {
            return;
        };

        let download_agent = self
            .download_agent_registry
            .get(&agent_data)
            .unwrap()
            .clone();

        let status = download_agent.status();

        // This download is already going
        if status != DownloadStatus::Queued {
            return;
        }

        // Ensure all others are marked as queued
        for agent in self.download_agent_registry.values() {
            if agent.metadata() != agent_data && agent.status() != DownloadStatus::Queued {
                agent.on_queued(&self.app_handle);
            }
        }

        info!("starting download for {agent_data:?}");
        self.active_control_flag = Some(download_agent.control_flag());

        let sender = self.sender.clone();

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        let app_handle = self.app_handle.clone();

        *download_thread_lock = Some(spawn(move || {
            loop {
                let download_result = match download_agent.download(&app_handle) {
                    // Ok(true) is for completed and exited properly
                    Ok(v) => v,
                    Err(e) => {
                        error!("download {:?} has error {}", download_agent.metadata(), &e);
                        download_agent.on_error(&app_handle, &e);
                        sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        return;
                    }
                };

                // If the download gets canceled
                // immediately return, on_cancelled gets called for us earlier
                if !download_result {
                    return;
                }

                if download_agent.control_flag().get() == DownloadThreadControlFlag::Stop {
                    return;
                }

                let validate_result = match download_agent.validate(&app_handle) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "download {:?} has validation error {}",
                            download_agent.metadata(),
                            &e
                        );
                        download_agent.on_error(&app_handle, &e);
                        sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        return;
                    }
                };

                if download_agent.control_flag().get() == DownloadThreadControlFlag::Stop {
                    return;
                }

                if validate_result {
                    download_agent.on_complete(&app_handle);
                    sender
                        .send(DownloadManagerSignal::Completed(download_agent.metadata()))
                        .unwrap();
                    sender.send(DownloadManagerSignal::UpdateUIQueue).unwrap();
                    return;
                }
            }
        }));

        self.set_status(DownloadManagerStatus::Downloading);
        let active_control_flag = self.active_control_flag.clone().unwrap();
        active_control_flag.set(DownloadThreadControlFlag::Go);
    }
    fn manage_stop_signal(&mut self) {
        debug!("got signal Stop");

        if let Some(active_control_flag) = self.active_control_flag.clone() {
            self.set_status(DownloadManagerStatus::Paused);
            active_control_flag.set(DownloadThreadControlFlag::Stop);
        }
    }
    fn manage_completed_signal(&mut self, meta: DownloadableMetadata) {
        debug!("got signal Completed");
        if let Some(interface) = self.download_queue.read().front()
            && interface == &meta
        {
            self.remove_and_cleanup_front_download(&meta);
        }

        self.push_ui_queue_update();
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }
    fn manage_error_signal(&mut self, error: ApplicationDownloadError) {
        debug!("got signal Error");
        if let Some(metadata) = self.download_queue.read().front()
            && let Some(current_agent) = self.download_agent_registry.get(metadata)
        {
            current_agent.on_error(&self.app_handle, &error);

            self.stop_and_wait_current_download();
            self.remove_and_cleanup_front_download(metadata);
        }
        self.push_ui_queue_update();
        self.set_status(DownloadManagerStatus::Error);
    }
    fn manage_cancel_signal(&mut self, meta: &DownloadableMetadata) {
        debug!("got signal Cancel");

        // If the current download is the one we're tryna cancel
        if let Some(current_metadata) = self.download_queue.read().front()
            && current_metadata == meta
            && let Some(current_download) = self.download_agent_registry.get(current_metadata)
        {
            self.set_status(DownloadManagerStatus::Paused);
            current_download.on_cancelled(&self.app_handle);
            self.stop_and_wait_current_download();

            self.download_queue.pop_front();

            self.cleanup_current_download();
            self.download_agent_registry.remove(meta);
            debug!("current download queue: {:?}", self.download_queue.read());
        }
        // else just cancel it
        else if let Some(download_agent) = self.download_agent_registry.get(meta) {
            let index = self.download_queue.get_by_meta(meta);
            if let Some(index) = index {
                download_agent.on_cancelled(&self.app_handle);
                let _ = self.download_queue.edit().remove(index).unwrap();
                let removed = self.download_agent_registry.remove(meta);
                debug!(
                    "removed {:?} from queue {:?}",
                    removed.map(|x| x.metadata()),
                    self.download_queue.read()
                );
            }
        }
        self.sender.send(DownloadManagerSignal::Go).unwrap();
        self.push_ui_queue_update();
    }
    fn push_ui_stats_update(&self, kbs: usize, time: usize) {
        let event_data = StatsUpdateEvent { speed: kbs, time };

        self.app_handle.emit("update_stats", event_data).unwrap();
    }
    fn push_ui_queue_update(&self) {
        let queue = &self.download_queue.read();
        let queue_objs = queue
            .iter()
            .map(|key| {
                let val = self.download_agent_registry.get(key).unwrap();
                QueueUpdateEventQueueData {
                    meta: DownloadableMetadata::clone(key),
                    status: val.status(),
                    progress: val.progress().get_progress(),
                    current: val.progress().sum(),
                    max: val.progress().get_max(),
                }
            })
            .collect();

        let event_data = QueueUpdateEvent { queue: queue_objs };
        self.app_handle.emit("update_queue", event_data).unwrap();
    }
}
