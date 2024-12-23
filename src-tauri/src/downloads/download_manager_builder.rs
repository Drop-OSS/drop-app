use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLockWriteGuard,
    },
    thread::{spawn, JoinHandle},
};

use log::{error, info};
use tauri::{AppHandle, Emitter};

use crate::{
    db::{Database, GameStatus, GameTransientStatus},
    library::{on_game_complete, GameUpdateEvent, QueueUpdateEvent, QueueUpdateEventQueueData},
    state::GameStatusManager,
    DB,
};

use super::{
    download_agent::{GameDownloadAgent, GameDownloadError},
    download_manager::{
        DownloadManager, DownloadManagerSignal, DownloadManagerStatus,
        GameDownloadAgentQueueStandin, GameDownloadStatus,
    },
    download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
    progress_object::ProgressObject,
    queue::Queue,
};

/*

Welcome to the download manager, the most overengineered, glorious piece of bullshit.

The download manager takes a queue of game_ids and their associated
GameDownloadAgents, and then, one-by-one, executes them. It provides an interface
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

This download queue does not actually own any of the GameDownloadAgents. It is
simply a id-based reference system. The actual Agents are stored in the
download_agent_registry HashMap, as ordering is no issue here. This is why
appending or removing from the download_queue must be done via signals.

Behold, my madness - quexeky

*/

// Refactored to consolidate this type. It's a monster.
pub type CurrentProgressObject = Arc<Mutex<Option<Arc<ProgressObject>>>>;

pub struct DownloadManagerBuilder {
    download_agent_registry: HashMap<String, Arc<Mutex<GameDownloadAgent>>>,
    download_queue: Queue,
    command_receiver: Receiver<DownloadManagerSignal>,
    sender: Sender<DownloadManagerSignal>,
    progress: CurrentProgressObject,
    status: Arc<Mutex<DownloadManagerStatus>>,
    app_handle: AppHandle,

    current_download_agent: Option<Arc<GameDownloadAgentQueueStandin>>, // Should be the only game download agent in the map with the "Go" flag
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

    fn set_game_status<F: FnOnce(&mut RwLockWriteGuard<'_, Database>, &String) -> ()>(
        &self,
        id: String,
        setter: F,
    ) {
        let mut db_handle = DB.borrow_data_mut().unwrap();
        setter(&mut db_handle, &id);
        drop(db_handle);
        DB.save().unwrap();

        let status = GameStatusManager::fetch_state(&id);

        self.app_handle
            .emit(
                &format!("update_game/{}", id),
                GameUpdateEvent {
                    game_id: id,
                    status,
                },
            )
            .unwrap();
    }

    fn push_manager_update(&self) {
        let queue = self.download_queue.read();
        let queue_objs: Vec<QueueUpdateEventQueueData> = queue
            .iter()
            .map(|interface| QueueUpdateEventQueueData {
                id: interface.id.clone(),
                status: interface.status.lock().unwrap().clone(),
                progress: interface.progress.get_progress(),
            })
            .collect();

        let event_data = QueueUpdateEvent { queue: queue_objs };
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

    fn sync_download_agent(&self) {}

    fn remove_and_cleanup_game(&mut self, game_id: &String) -> Arc<Mutex<GameDownloadAgent>> {
        self.download_queue.pop_front();
        let download_agent = self.download_agent_registry.remove(game_id).unwrap();
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
                DownloadManagerSignal::Completed(game_id) => {
                    self.manage_completed_signal(game_id);
                }
                DownloadManagerSignal::Queue(game_id, version, target_download_dir) => {
                    self.manage_queue_signal(game_id, version, target_download_dir);
                }
                DownloadManagerSignal::Error(e) => {
                    self.manage_error_signal(e);
                }
                DownloadManagerSignal::Cancel => {
                    self.manage_cancel_signal();
                }
                DownloadManagerSignal::Update => {
                    self.push_manager_update();
                }
                DownloadManagerSignal::Finish => {
                    self.stop_and_wait_current_download();
                    return Ok(());
                }
                DownloadManagerSignal::Remove(game_id) => {
                    self.manage_remove_game(game_id);
                }
            };
        }
    }

    fn manage_remove_game(&mut self, game_id: String) {
        if let Some(current_download) = &self.current_download_agent {
            if current_download.id == game_id {
                self.manage_cancel_signal();
            }
        }

        let index = self.download_queue.get_by_id(game_id.clone()).unwrap();
        let mut queue_handle = self.download_queue.edit();
        queue_handle.remove(index);
        self.set_game_status(game_id, |db_handle, id| {
            db_handle.games.transient_statuses.remove(id);
        });
        drop(queue_handle);

        if self.current_download_agent.is_none() {
            self.manage_go_signal();
        }

        self.push_manager_update();
    }

    fn manage_stop_signal(&mut self) {
        info!("Got signal 'Stop'");
        self.set_status(DownloadManagerStatus::Paused);
        if let Some(active_control_flag) = self.active_control_flag.clone() {
            active_control_flag.set(DownloadThreadControlFlag::Stop);
        }
    }

    fn manage_completed_signal(&mut self, game_id: String) {
        info!("Got signal 'Completed'");
        if let Some(interface) = &self.current_download_agent {
            // When if let chains are stabilised, combine these two statements
            if interface.id == game_id {
                info!("Popping consumed data");
                let download_agent = self.remove_and_cleanup_game(&game_id);
                let download_agent_lock = download_agent.lock().unwrap();

                let version = download_agent_lock.version.clone();
                let install_dir = download_agent_lock.base_dir.clone();

                drop(download_agent_lock);

                if let Err(error) =
                    on_game_complete(game_id, version, install_dir, &self.app_handle)
                {
                    self.sender
                        .send(DownloadManagerSignal::Error(
                            GameDownloadError::Communication(error),
                        ))
                        .unwrap();
                }
            }
        }
        self.sender.send(DownloadManagerSignal::Update).unwrap();
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }

    fn manage_queue_signal(&mut self, id: String, version: String, target_download_dir: usize) {
        info!("Got signal Queue");
        let download_agent = Arc::new(Mutex::new(GameDownloadAgent::new(
            id.clone(),
            version,
            target_download_dir,
            self.sender.clone(),
        )));
        let download_agent_lock = download_agent.lock().unwrap();

        let agent_status = GameDownloadStatus::Queued;
        let interface_data = GameDownloadAgentQueueStandin {
            id: id.clone(),
            status: Mutex::new(agent_status),
            progress: download_agent_lock.progress.clone(),
        };
        let version_name = download_agent_lock.version.clone();

        drop(download_agent_lock);

        self.download_agent_registry
            .insert(interface_data.id.clone(), download_agent);
        self.download_queue.append(interface_data);

        self.set_game_status(id, |db, id| {
            db.games.transient_statuses.insert(
                id.to_string(),
                GameTransientStatus::Downloading { version_name },
            );
        });
        self.sender.send(DownloadManagerSignal::Update).unwrap();
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

        let version_name = download_agent_lock.version.clone();

        let progress_object = download_agent_lock.progress.clone();
        *self.progress.lock().unwrap() = Some(progress_object);

        let active_control_flag = download_agent_lock.control_flag.clone();
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

        // Set status for games
        for queue_game in self.download_queue.read() {
            let mut status_handle = queue_game.status.lock().unwrap();
            if queue_game.id == agent_data.id {
                *status_handle = GameDownloadStatus::Downloading;
            } else {
                *status_handle = GameDownloadStatus::Queued;
            }
            drop(status_handle);
        }

        // Set flags for download manager
        active_control_flag.set(DownloadThreadControlFlag::Go);
        self.set_status(DownloadManagerStatus::Downloading);
        self.set_game_status(agent_data.id.clone(), |db, id| {
            db.games.transient_statuses.insert(
                id.to_string(),
                GameTransientStatus::Downloading { version_name },
            );
        });

        self.sender.send(DownloadManagerSignal::Update).unwrap();
    }
    fn manage_error_signal(&mut self, error: GameDownloadError) {
        let current_status = self.current_download_agent.clone().unwrap();

        self.remove_and_cleanup_game(&current_status.id); // Remove all the locks and shit

        let mut lock = current_status.status.lock().unwrap();
        *lock = GameDownloadStatus::Error;
        self.set_status(DownloadManagerStatus::Error(error));

        let game_id = current_status.id.clone();
        self.set_game_status(game_id, |db_handle, id| {
            db_handle.games.transient_statuses.remove(id);
        });

        self.sender.send(DownloadManagerSignal::Update).unwrap();
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
