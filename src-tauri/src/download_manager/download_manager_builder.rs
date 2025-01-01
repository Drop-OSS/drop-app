use std::{
    collections::HashMap,
    fs::remove_dir_all,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLockWriteGuard,
    },
    thread::{spawn, JoinHandle},
};

use log::{error, info};
use tauri::{AppHandle, Emitter};

use crate::{
    db::{set_application_status, ApplicationStatus, ApplicationTransientStatus, Database}, download_manager::{download_manager::{DownloadStatus, DownloadType}, generate_downloadable::generate_downloadable}, downloads::download_agent::{self, GameDownloadAgent}, library::{
        on_game_complete, push_application_update, QueueUpdateEvent,
        QueueUpdateEventQueueData, StatsUpdateEvent,
    }, state::DownloadStatusManager, DB
};

use super::{application_download_error::ApplicationDownloadError, download_manager::{DownloadManager, DownloadManagerSignal, DownloadManagerStatus}, download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag}, downloadable::Downloadable, downloadable_metadata::DownloadableMetadata, progress_object::ProgressObject, queue::Queue};

pub struct DownloadableQueueStandin {
    pub id: String,
    pub status: Mutex<DownloadStatus>,
    pub progress: Arc<ProgressObject>,
}
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
simply a id-based reference system. The actual Agents are stored in the
download_agent_registry HashMap, as ordering is no issue here. This is why
appending or removing from the download_queue must be done via signals.

Behold, my madness - quexeky

*/

pub struct DownloadManagerBuilder {
    download_agent_registry: HashMap<Arc<DownloadableMetadata>, DownloadAgent>,
    download_queue: Queue,
    command_receiver: Receiver<DownloadManagerSignal>,
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
            command_receiver,
            status: status.clone(),
            sender: command_sender.clone(),
            progress: active_progress.clone(),
            app_handle,

            current_download_agent: None,
            current_download_thread: Mutex::new(None),
            active_control_flag: None,
        };

        let terminator = spawn(|| manager.manage_queue());

        DownloadManager::new(terminator, queue, active_progress, command_sender)
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
                DownloadManagerSignal::Completed(id) => {
                    self.manage_completed_signal(id);
                }
                DownloadManagerSignal::Queue(download_agent) => {
                    self.manage_queue_signal(download_agent);
                }
                DownloadManagerSignal::Error(e) => {
                    self.manage_error_signal(e);
                }
                DownloadManagerSignal::Cancel => {
                    self.manage_cancel_signal();
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
                DownloadManagerSignal::Remove(id) => {
                    self.manage_remove_download_from_queue(id);
                }
                DownloadManagerSignal::Uninstall(id) => {
                    self.uninstall_application(id);
                }
            };
        }
    }
    fn manage_queue_signal(&mut self, download_agent: DownloadAgent) {
        info!("Got signal Queue");
        let meta = download_agent.metadata();

        if self.download_queue.exists(meta.clone()) {
            info!("Download with same ID already exists");
            return;
        }

        let download_agent = generate_downloadable(meta.clone());
        download_agent.on_initialised(&self.app_handle);
        self.download_queue.append(meta.clone());
        self.download_agent_registry.insert(meta, download_agent);

        self.sender.send(DownloadManagerSignal::UpdateUIQueue).unwrap();
    }

    fn manage_go_signal(&mut self) {
        info!("Got signal Go");
        if !(self.download_agent_registry.is_empty()) { return; }

        if self.current_download_agent.is_some() { return; }

        info!("Current download queue: {:?}", self.download_queue.read());

        // Should always be Some if the above two statements keep going
        let agent_data = self.download_queue.read().front().unwrap().clone();

        info!("Starting download for {:?}", agent_data);

        let download_agent = self
            .download_agent_registry
            .get(&agent_data)
            .unwrap()
            .clone();

        self.active_control_flag = Some(download_agent.control_flag());
        self.current_download_agent = Some(download_agent.clone());

        let sender = self.sender.clone();

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        let app_handle = self.app_handle.clone();

        *download_thread_lock = Some(spawn(move || {
            match download_agent.download() {
                // Ok(true) is for completed and exited properly
                Ok(true) => {
                    download_agent.on_complete(&app_handle);
                },
                // Ok(false) is for incomplete but exited properly
                Ok(false) => {
                    download_agent.on_incomplete(&app_handle);
                },
                Err(e) => {
                    download_agent.on_error(&app_handle);
                    error!("error while managing download: {}", e);
                    sender.send(DownloadManagerSignal::Error(e)).unwrap();
                },
            }
        }));

        let active_control_flag = self.active_control_flag.clone().unwrap();
        active_control_flag.set(DownloadThreadControlFlag::Go);

    }
}
/*
// Refactored to consolidate this type. It's a monster.
pub type DownloadAgent = Arc<Mutex<Box<dyn Downloadable + Send + Sync>>>;


impl DownloadManagerBuilder {
    fn manage_go_signal(&mut self) {
        if !(!self.download_agent_registry.is_empty() && !self.download_queue.empty()) {
            return;
        }

        if self.current_download_agent.is_some() {
            info!("skipping go signal due to existing download job");
            return;
        }

        info!("current download queue: {:?}", self.download_queue.read());
        let agent_data = self.download_queue.read().front().unwrap().clone();
        info!("starting download for {}", agent_data.id.clone());
        let download_agent = self
            .download_agent_registry
            .get(&agent_data.id)
            .unwrap()
            .clone();
        let download_agent_lock = download_agent.lock().unwrap();
        self.current_download_agent = Some(agent_data);
        // Cloning option should be okay because it only clones the Arc inside, not the AgentInterfaceData
        let agent_data = self.current_download_agent.clone().unwrap();

        let version_name = download_agent_lock.version().clone();

        let progress_object = download_agent_lock.progress();
        *self.progress.lock().unwrap() = Some(progress_object);

        let active_control_flag = download_agent_lock.control_flag();
        self.active_control_flag = Some(active_control_flag.clone());

        let sender = self.sender.clone();

        drop(download_agent_lock);

        info!("Spawning download");
        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        *download_thread_lock = Some(spawn(move || {
            let mut download_agent_lock = download_agent.lock().unwrap();
            match download_agent_lock.download() {
                // Returns once we've exited the download
                // (not necessarily completed)
                // The download agent will fire the completed event for us
                Ok(_) => {}
                // If an error occurred while *starting* the download
                Err(err) => {
                    error!("error while managing download: {}", err);
                    sender.send(DownloadManagerSignal::Error(err)).unwrap();
                }
            };
            drop(download_agent_lock);
        }));

        // Set status for applications
        for queue_application in self.download_queue.read() {
            let mut status_handle = queue_application.status.lock().unwrap();
            if queue_application.id == agent_data.id {
                *status_handle = DownloadStatus::Downloading;
            } else {
                *status_handle = DownloadStatus::Queued;
            }
            drop(status_handle);
        }

        // Set flags for download manager
        active_control_flag.set(DownloadThreadControlFlag::Go);
        self.set_status(DownloadManagerStatus::Downloading);
        self.set_application_status(agent_data.id.clone(), |db, id| {
            db.applications.transient_statuses.insert(
                id.to_string(),
                ApplicationTransientStatus::Downloading { version_name },
            );
        });

        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }

    fn manage_queue_signal(&mut self, id: String, version: String, target_download_dir: usize) {
        info!("Got signal Queue");

        if let Some(index) = self.download_queue.get_by_id(id.clone()) {
            // Should always give us a value
            if let Some(download_agent) = self.download_agent_registry.get(&id) {
                let download_agent_handle = download_agent.lock().unwrap();
                if download_agent_handle.version() == version {
                    info!("Application with same version already queued, skipping");
                    return;
                }
                // If it's not the same, we want to cancel the current one, and then add the new one
                drop(download_agent_handle);

                self.manage_remove_download_from_queue(id.clone());
            }
        }

        let download_agent = Arc::new(Mutex::new(DownloadType::Game.generate(
            id.clone(),
            version,
            target_download_dir,
            self.sender.clone(),
        )));
        let download_agent_lock = download_agent.lock().unwrap();

        let agent_status = DownloadStatus::Queued;
        let interface_data = DownloadableQueueStandin {
            id: id.clone(),
            status: Mutex::new(agent_status),
            progress: download_agent_lock.progress()
        };
        let version_name = download_agent_lock.version().clone();

        drop(download_agent_lock);

        self.download_agent_registry
            .insert(interface_data.id.clone(), download_agent);
        self.download_queue.append(interface_data);

        self.set_application_status(id, |db, id| {
            db.applications.transient_statuses.insert(
                id.to_string(),
                ApplicationTransientStatus::Downloading { version_name },
            );
        });
        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }

    fn push_ui_stats_update(&self, kbs: usize, time: usize) {
        let event_data = StatsUpdateEvent { speed: kbs, time };

        self.app_handle.emit("update_stats", event_data).unwrap();
    }

    fn push_ui_queue_update(&self) {
        let queue = self.download_queue.read();
        let queue_objs: Vec<QueueUpdateEventQueueData> = queue
            .iter()
            .map(|interface| QueueUpdateEventQueueData {
                id: interface.id.clone(),
                status: interface.status.lock().unwrap().clone(),
                progress: interface.progress.get_progress(),
            })
            .collect();

        let status_handle = self.status.lock().unwrap();
        let status = status_handle.clone();
        drop(status_handle);

        let event_data = QueueUpdateEvent {
            queue: queue_objs,
            status,
        };
        self.app_handle.emit("update_queue", event_data).unwrap();
    }

    fn stop_and_wait_current_download(&self) {
        self.set_status(DownloadManagerStatus::Paused);
        if let Some(current_flag) = &self.active_control_flag {
            current_flag.set(DownloadThreadControlFlag::Stop);
        }

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        if let Some(current_download_thread) = download_thread_lock.take() {
            current_download_thread.join().unwrap();
        }
        drop(download_thread_lock);
    }

    fn remove_and_cleanup_front_download(&mut self, id: &String) -> DownloadAgent {
        self.download_queue.pop_front();
        let download_agent = self.download_agent_registry.remove(id).unwrap();
        self.cleanup_current_download();
        download_agent
    }

    // CAREFUL WITH THIS FUNCTION
    // Make sure the download thread is terminated
    fn cleanup_current_download(&mut self) {
        self.active_control_flag = None;
        *self.progress.lock().unwrap() = None;
        self.current_download_agent = None;

        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        *download_thread_lock = None;
        drop(download_thread_lock);
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
                DownloadManagerSignal::Completed(id) => {
                    self.manage_completed_signal(id);
                }
                DownloadManagerSignal::Queue(id, version, target_download_dir) => {
                    self.manage_queue_signal(id, version, target_download_dir);
                }
                DownloadManagerSignal::Error(e) => {
                    self.manage_error_signal(e);
                }
                DownloadManagerSignal::Cancel => {
                    self.manage_cancel_signal();
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
                DownloadManagerSignal::Remove(id) => {
                    self.manage_remove_download_from_queue(id);
                }
                DownloadManagerSignal::Uninstall(id) => {
                    self.uninstall_application(id);
                }
            };
        }
    }

    fn uninstall_application(&mut self, id: String) {
        // Removes the download if it's in the queue
        self.manage_remove_download_from_queue(id.clone());

        let mut db_handle = DB.borrow_data_mut().unwrap();
        db_handle
            .applications
            .transient_statuses
            .entry(id.clone())
            .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});
        push_application_update(
            &self.app_handle,
            id.clone(),
            (None, Some(ApplicationTransientStatus::Uninstalling {})),
        );

        let previous_state = db_handle.applications.statuses.get(&id).cloned();
        if previous_state.is_none() {
            info!("uninstall job doesn't have previous state, failing silently");
            return;
        }
        let previous_state = previous_state.unwrap();
        if let Some((_version_name, install_dir)) = match previous_state {
            ApplicationStatus::Installed {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            ApplicationStatus::SetupRequired {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            _ => None,
        } {
            db_handle
                .applications
                .transient_statuses
                .entry(id.clone())
                .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});
            drop(db_handle);

            let sender = self.sender.clone();
            let app_handle = self.app_handle.clone();
            spawn(move || match remove_dir_all(install_dir) {
                Err(e) => {
                    sender
                        .send(DownloadManagerSignal::Error(ApplicationDownloadError::IoError(
                            e.kind(),
                        )))
                        .unwrap();
                }
                Ok(_) => {
                    let mut db_handle = DB.borrow_data_mut().unwrap();
                    db_handle.applications.transient_statuses.remove(&id);
                    db_handle
                        .applications
                        .statuses
                        .entry(id.clone())
                        .and_modify(|e| *e = ApplicationStatus::Remote {});
                    drop(db_handle);
                    DB.save().unwrap();

                    info!("uninstalled {}", id);

                    push_application_update(&app_handle, id, (Some(ApplicationStatus::Remote {}), None));
                }
            });
        }
    }

    fn manage_remove_download_from_queue(&mut self, id: DownloadableMetadata) {
        if let Some(current_download) = &self.current_download_agent {
            if current_download.lock().unwrap().metadata() == id {
                self.manage_cancel_signal();
            }
        }

        let index = self.download_queue.get_by_id(id.clone());
        if let Some(index) = index {
            let mut queue_handle = self.download_queue.edit();
            queue_handle.remove(index);
            set_application_status(&self.app_handle, id, |db_handle, id| {
                db_handle.applications.transient_statuses.remove(id);
            });
            drop(queue_handle);
        }

        if self.current_download_agent.is_none() {
            self.manage_go_signal();
        }

        self.push_ui_queue_update();
    }

    fn manage_stop_signal(&mut self) {
        info!("Got signal 'Stop'");
        self.set_status(DownloadManagerStatus::Paused);
        if let Some(active_control_flag) = self.active_control_flag.clone() {
            active_control_flag.set(DownloadThreadControlFlag::Stop);
        }
    }

    fn manage_completed_signal(&mut self, id: String) {
        info!("Got signal 'Completed'");
        if let Some(interface) = &self.current_download_agent {
            if interface.id == id {
                info!("Popping consumed data");
                let download_agent = self.remove_and_cleanup_front_download(&id);
                let download_agent_lock = download_agent.lock().unwrap();

                drop(download_agent_lock);

                if let Err(error) =
                    self.current_download_agent.on_complete(&self.app_handle);
                {
                    self.sender
                        .send(DownloadManagerSignal::Error(
                            ApplicationDownloadError::Communication(error),
                        ))
                        .unwrap();
                }
            }
        }
        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }


    fn manage_go_signal(&mut self) {
        if !(!self.download_agent_registry.is_empty() && !self.download_queue.empty()) {
            return;
        }

        if self.current_download_agent.is_some() {
            info!("skipping go signal due to existing download job");
            return;
        }

        info!("current download queue: {:?}", self.download_queue.read());
        let agent_data = self.download_queue.read().front().unwrap().clone();
        info!("starting download for {}", agent_data.id.clone());
        let download_agent = self
            .download_agent_registry
            .get(&agent_data.id)
            .unwrap()
            .clone();
        let download_agent_lock = download_agent.lock().unwrap();
        self.current_download_agent = Some(agent_data);
        // Cloning option should be okay because it only clones the Arc inside, not the AgentInterfaceData
        let agent_data = self.current_download_agent.clone().unwrap();

        let version_name = download_agent_lock.version().clone();

        let progress_object = download_agent_lock.progress();
        *self.progress.lock().unwrap() = Some(progress_object);

        let active_control_flag = download_agent_lock.control_flag();
        self.active_control_flag = Some(active_control_flag.clone());

        let sender = self.sender.clone();

        drop(download_agent_lock);

        info!("Spawning download");
        let mut download_thread_lock = self.current_download_thread.lock().unwrap();
        *download_thread_lock = Some(spawn(move || {
            let mut download_agent_lock = download_agent.lock().unwrap();
            match download_agent_lock.download() {
                // Returns once we've exited the download
                // (not necessarily completed)
                // The download agent will fire the completed event for us
                Ok(_) => {}
                // If an error occurred while *starting* the download
                Err(err) => {
                    error!("error while managing download: {}", err);
                    sender.send(DownloadManagerSignal::Error(err)).unwrap();
                }
            };
            drop(download_agent_lock);
        }));

        // Set status for applications
        for queue_application in self.download_queue.read() {
            let mut status_handle = queue_application.status.lock().unwrap();
            if queue_application.id == agent_data.id {
                *status_handle = DownloadStatus::Downloading;
            } else {
                *status_handle = DownloadStatus::Queued;
            }
            drop(status_handle);
        }

        // Set flags for download manager
        active_control_flag.set(DownloadThreadControlFlag::Go);
        self.set_status(DownloadManagerStatus::Downloading);
        self.set_application_status(agent_data.id.clone(), |db, id| {
            db.applications.transient_statuses.insert(
                id.to_string(),
                ApplicationTransientStatus::Downloading { version_name },
            );
        });

        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }
    fn manage_error_signal(&mut self, error: ApplicationDownloadError) {
        let current_status = self.current_download_agent.clone().unwrap();

        self.stop_and_wait_current_download();
        self.remove_and_cleanup_front_download(&current_status.id); // Remove all the locks and shit, and remove from queue

        self.app_handle
            .emit("download_error", error.to_string())
            .unwrap();

        let mut lock = current_status.status.lock().unwrap();
        *lock = DownloadStatus::Error;
        self.set_status(DownloadManagerStatus::Error(error));

        let id = current_status.id.clone();
        self.set_application_status(id, |db_handle, id| {
            db_handle.applications.transient_statuses.remove(id);
        });

        self.sender
            .send(DownloadManagerSignal::UpdateUIQueue)
            .unwrap();
    }
    fn manage_cancel_signal(&mut self) {
        self.stop_and_wait_current_download();

        info!("cancel waited for download to finish");

        self.cleanup_current_download();
    }
    fn set_status(&self, status: DownloadManagerStatus) {
        *self.status.lock().unwrap() = status;
    }
}
*/