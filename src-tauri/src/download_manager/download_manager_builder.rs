use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};

use ::futures::future::join_all;
use log::{debug, error, info, warn};
use tauri::{AppHandle, Emitter};
use tokio::{spawn, sync::Mutex, task::JoinHandle};

use crate::{
    database::models::data::DownloadableMetadata,
    error::application_download_error::ApplicationDownloadError,
    games::library::{QueueUpdateEvent, QueueUpdateEventQueueData, StatsUpdateEvent},
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

/*

Welcome to the download manager, the most overengineered, glorious piece of bullshit.

The download manager takes a queue of ids and their associated
DownloadAgents, and then, one-by-one, executes them. It provides an interface
to interact with the currently downloading agent, and manage the queue.

When the DownloadManager is initialised, it is designed to provide a reference
which can be used to provide some instructions (the DownloadManagerInterface),
but other than that, it runs without any sort of interruptions.

It does this by opening up two data structures. Primarily is the command_receiver,
and mpsc (multi-channel-single-producer) which allows commands to be sent from
the Interface, and queued up for the Manager to process.

These have been mapped in the DownloadManagerSignal docs.

The other way to interact with the DownloadManager is via the donwload_queue,
which is just a collection of ids which may be rearranged to suit
whichever download queue order is required.

+----------------------------------------------------------------------------+
| DO NOT ATTEMPT TO ADD OR REMOVE FROM THE QUEUE WITHOUT USING SIGNALS!!     |
| THIS WILL CAUSE A DESYNC BETWEEN THE DOWNLOAD AGENT REGISTRY AND THE QUEUE |
| WHICH HAS NOT BEEN ACCOUNTED FOR                                           |
+----------------------------------------------------------------------------+

This download queue does not actually own any of the DownloadAgents. It is
simply an id-based reference system. The actual Agents are stored in the
download_agent_registry HashMap, as ordering is no issue here. This is why
appending or removing from the download_queue must be done via signals.

Behold, my madness - quexeky

*/

pub struct DownloadManagerBuilder {
    download_agent_registry: HashMap<DownloadableMetadata, DownloadAgent>,
    download_queue: Queue,
    command_receiver: Mutex<Receiver<DownloadManagerSignal>>,
    sender: Sender<DownloadManagerSignal>,
    progress: CurrentProgressObject,
    status: Arc<Mutex<DownloadManagerStatus>>,
    app_handle: AppHandle,

    current_download_agent: Option<DownloadAgent>, // Should be the only download agent in the map with the "Go" flag
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
            command_receiver: Mutex::new(command_receiver),
            status: status.clone(),
            sender: command_sender.clone(),
            progress: active_progress.clone(),
            app_handle,

            current_download_agent: None,
            current_download_thread: Mutex::new(None),
            active_control_flag: None,
        };

        let terminator = spawn(async{ manager.manage_queue().await });

        DownloadManager::new(terminator, queue, active_progress, command_sender)
    }

    async fn set_status(&self, status: DownloadManagerStatus) {
        *self.status.lock().await = status;
    }

    async fn remove_and_cleanup_front_download(&mut self, meta: &DownloadableMetadata) -> DownloadAgent {
        self.download_queue.pop_front();
        let download_agent = self.download_agent_registry.remove(meta).unwrap();
        self.cleanup_current_download().await;
        download_agent
    }

    // CAREFUL WITH THIS FUNCTION
    // Make sure the download thread is terminated
    async fn cleanup_current_download(&mut self) {
        self.active_control_flag = None;
        *self.progress.lock().await = None;
        self.current_download_agent = None;

        let mut download_thread_lock = self.current_download_thread.lock().await;
        *download_thread_lock = None;
        drop(download_thread_lock);
    }

    async fn stop_and_wait_current_download(&self) {
        self.set_status(DownloadManagerStatus::Paused).await;
        if let Some(current_flag) = &self.active_control_flag {
            current_flag.set(DownloadThreadControlFlag::Stop);
        }

        let mut download_thread_lock = self.current_download_thread.lock().await;
        if let Some(current_download_thread) = download_thread_lock.take() {
            current_download_thread.await.unwrap();
        }
    }

    async fn manage_queue(mut self) -> Result<(), ()> {
        loop {
            let signal = match self.command_receiver.lock().await.recv() {
                Ok(signal) => signal,
                Err(_) => return Err(()),
            };

            match signal {
                DownloadManagerSignal::Go => {
                    self.manage_go_signal().await;
                }
                DownloadManagerSignal::Stop => {
                    self.manage_stop_signal().await;
                }
                DownloadManagerSignal::Completed(meta) => {
                    self.manage_completed_signal(meta).await;
                }
                DownloadManagerSignal::Queue(download_agent) => {
                    self.manage_queue_signal(download_agent).await;
                }
                DownloadManagerSignal::Error(e) => {
                    self.manage_error_signal(e).await;
                }
                DownloadManagerSignal::UpdateUIQueue => {
                    self.push_ui_queue_update().await;
                }
                DownloadManagerSignal::UpdateUIStats(kbs, time) => {
                    self.push_ui_stats_update(kbs, time);
                }
                DownloadManagerSignal::Finish => {
                    self.stop_and_wait_current_download().await;
                    return Ok(());
                }
                DownloadManagerSignal::Cancel(meta) => {
                    self.manage_cancel_signal(&meta).await;
                }
            };
        }
    }
    async fn manage_queue_signal(&mut self, download_agent: DownloadAgent) {
        debug!("got signal Queue");
        let meta = download_agent.metadata();

        debug!("queue metadata: {meta:?}");

        if self.download_queue.exists(meta.clone()) {
            warn!("download with same ID already exists");
            return;
        }

        download_agent.on_initialised(&self.app_handle).await;
        self.download_queue.append(meta.clone());
        self.download_agent_registry.insert(meta, download_agent);

        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }

    async fn manage_go_signal(&mut self) {
        debug!("got signal Go");
        if self.download_agent_registry.is_empty() {
            debug!(
                "Download agent registry: {:?}",
                self.download_agent_registry.len()
            );
            return;
        }

        if self.current_download_agent.is_some()
            && self.download_queue.read().front().unwrap()
                == &self.current_download_agent.as_ref().unwrap().metadata()
        {
            debug!(
                "Current download agent: {:?}",
                self.current_download_agent.as_ref().unwrap().metadata()
            );
            return;
        }

        debug!("current download queue: {:?}", self.download_queue.read());

        // Should always be Some if the above two statements keep going
        let agent_data = self.download_queue.read().front().unwrap().clone();

        info!("starting download for {agent_data:?}");

        let download_agent = self
            .download_agent_registry
            .get(&agent_data)
            .unwrap()
            .clone();

        self.active_control_flag = Some(download_agent.control_flag().await);
        self.current_download_agent = Some(download_agent.clone());

        let sender = self.sender.clone();

        let mut download_thread_lock = self.current_download_thread.lock().await;
        let app_handle = self.app_handle.clone();

        *download_thread_lock = Some(spawn(async move {
            match download_agent.download(&app_handle).await {
                // Ok(true) is for completed and exited properly
                Ok(true) => {
                    debug!("download {:?} has completed", download_agent.metadata());
                    match download_agent.validate().await {
                        Ok(true) => {
                            download_agent.on_complete(&app_handle).await;
                            sender
                                .send(DownloadManagerSignal::Completed(download_agent.metadata()))
                                .unwrap();
                        }
                        Ok(false) => {
                            download_agent.on_incomplete(&app_handle).await;
                        }
                        Err(e) => {
                            error!(
                                "download {:?} has validation error {}",
                                download_agent.metadata(),
                                &e
                            );
                            download_agent.on_error(&app_handle, &e).await;
                            sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        }
                    }
                }
                // Ok(false) is for incomplete but exited properly
                Ok(false) => {
                    debug!("Donwload agent finished incomplete");
                    download_agent.on_incomplete(&app_handle).await;
                }
                Err(e) => {
                    error!("download {:?} has error {}", download_agent.metadata(), &e);
                    download_agent.on_error(&app_handle, &e).await;
                    sender.send(DownloadManagerSignal::Error(e)).unwrap();
                }
            }
            sender.send(DownloadManagerSignal::UpdateUIQueue).unwrap();
        }));

        self.set_status(DownloadManagerStatus::Downloading).await;
        let active_control_flag = self.active_control_flag.clone().unwrap();
        active_control_flag.set(DownloadThreadControlFlag::Go);
    }
    async fn manage_stop_signal(&mut self) {
        debug!("got signal Stop");

        if let Some(active_control_flag) = self.active_control_flag.clone() {
            self.set_status(DownloadManagerStatus::Paused).await;
            active_control_flag.set(DownloadThreadControlFlag::Stop);
        }
    }
    async fn manage_completed_signal(&mut self, meta: DownloadableMetadata) {
        debug!("got signal Completed");
        if let Some(interface) = &self.current_download_agent
            && interface.metadata() == meta
        {
            self.remove_and_cleanup_front_download(&meta).await;
        }

        self.push_ui_queue_update().await;
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }
    async fn manage_error_signal(&mut self, error: ApplicationDownloadError) {
        debug!("got signal Error");
        if let Some(current_agent) = self.current_download_agent.clone() {
            current_agent.on_error(&self.app_handle, &error).await;

            self.stop_and_wait_current_download().await;
            self.remove_and_cleanup_front_download(&current_agent.metadata()).await;
        }
        self.set_status(DownloadManagerStatus::Error).await;
    }
    async fn manage_cancel_signal(&mut self, meta: &DownloadableMetadata) {
        debug!("got signal Cancel");

        if let Some(current_download) = &self.current_download_agent {
            if &current_download.metadata() == meta {
                self.set_status(DownloadManagerStatus::Paused).await;
                current_download.on_cancelled(&self.app_handle).await;
                self.stop_and_wait_current_download().await;

                self.download_queue.pop_front();

                self.cleanup_current_download().await;
                debug!("current download queue: {:?}", self.download_queue.read());
            }
            // TODO: Collapse these two into a single if statement somehow
            else if let Some(download_agent) = self.download_agent_registry.get(meta) {
                let index = self.download_queue.get_by_meta(meta);
                if let Some(index) = index {
                    download_agent.on_cancelled(&self.app_handle).await;
                    let _ = self.download_queue.edit().remove(index).unwrap();
                    let removed = self.download_agent_registry.remove(meta);
                    debug!(
                        "removed {:?} from queue {:?}",
                        removed.map(|x| x.metadata()),
                        self.download_queue.read()
                    );
                }
            }
        } else if let Some(download_agent) = self.download_agent_registry.get(meta) {
            let index = self.download_queue.get_by_meta(meta);
            if let Some(index) = index {
                download_agent.on_cancelled(&self.app_handle).await;
                let _ = self.download_queue.edit().remove(index).unwrap();
                let removed = self.download_agent_registry.remove(meta);
                debug!(
                    "removed {:?} from queue {:?}",
                    removed.map(|x| x.metadata()),
                    self.download_queue.read()
                );
            }
        }
        self.push_ui_queue_update().await;
    }
    fn push_ui_stats_update(&self, kbs: usize, time: usize) {
        let event_data = StatsUpdateEvent { speed: kbs, time };

        self.app_handle.emit("update_stats", event_data).unwrap();
    }
    async fn push_ui_queue_update(&self) {
        let queue = &self.download_queue.read();
        let queue_objs = join_all(queue
            .iter()
            .map(async |key| {
                let val = self.download_agent_registry.get(key).unwrap();
                QueueUpdateEventQueueData {
                    meta: DownloadableMetadata::clone(key),
                    status: val.status().await,
                    progress: val.progress().await.get_progress(),
                    current: val.progress().await.sum(),
                    max: val.progress().await.get_max(),
                }
            })).await;

        let event_data = QueueUpdateEvent { queue: queue_objs };
        self.app_handle.emit("update_queue", event_data).unwrap();
    }
}
