use std::{
    collections::{HashMap, VecDeque},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::spawn,
};

use log::info;

use super::{
    download_agent::{GameDownloadAgent, GameDownloadError},
    download_manager_interface::{AgentInterfaceData, DownloadManagerInterface},
    download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
    progress_object::ProgressObject,
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

pub struct DownloadManager {
    download_agent_registry: HashMap<String, Arc<GameDownloadAgent>>,
    download_queue: Arc<Mutex<VecDeque<Arc<AgentInterfaceData>>>>,
    command_receiver: Receiver<DownloadManagerSignal>,
    sender: Sender<DownloadManagerSignal>,
    progress: Arc<Mutex<Option<ProgressObject>>>,
    status: Arc<Mutex<DownloadManagerStatus>>,

    current_game_interface: Option<Arc<AgentInterfaceData>>, // Should be the only game download agent in the map with the "Go" flag
    active_control_flag: Option<DownloadThreadControl>,
}
pub enum DownloadManagerSignal {
    /// Resumes (or starts) the DownloadManager
    Go,
    /// Pauses the DownloadManager
    Stop,
    /// Called when a GameDownloadAgent has finished.
    /// Triggers the next download cycle to begin
    Completed(String),
    /// Generates and appends a GameDownloadAgent
    /// to the registry and queue
    Queue(String, String, usize),
    /// Tells the Manager to stop the current
    /// download and return
    Finish,
    /// Any error which occurs in the agent
    Error(GameDownloadError),
}
pub enum DownloadManagerStatus {
    Downloading,
    Paused,
    Empty,
    Error(GameDownloadError),
}
#[derive(Clone)]
pub enum GameDownloadStatus {
    Downloading,
    Paused,
    Uninitialised,
    Error(GameDownloadError),
}

impl DownloadManager {
    pub fn generate() -> DownloadManagerInterface {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let (command_sender, command_receiver) = channel();
        let active_progress = Arc::new(Mutex::new(None));
        let status = Arc::new(Mutex::new(DownloadManagerStatus::Empty));

        let manager = Self {
            download_agent_registry: HashMap::new(),
            download_queue: queue.clone(),
            command_receiver,
            current_game_interface: None,
            active_control_flag: None,
            status: status.clone(),
            sender: command_sender.clone(),
            progress: active_progress.clone(),
        };

        let terminator = spawn(|| manager.manage_queue());

        DownloadManagerInterface::new(terminator, queue, active_progress, command_sender)
    }

    fn manage_queue(mut self) -> Result<(), ()> {
        loop {
            let signal = match self.command_receiver.recv() {
                Ok(signal) => signal,
                Err(e) => return Err(()),
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
                DownloadManagerSignal::Finish => {
                    if let Some(active_control_flag) = self.active_control_flag {
                        active_control_flag.set(DownloadThreadControlFlag::Stop)
                    }
                    return Ok(());
                }
                DownloadManagerSignal::Error(game_download_error) => {
                    self.manage_error_signal(game_download_error);
                }
            };
        }
    }

    fn manage_stop_signal(&mut self) {
        info!("Got signal 'Stop'");
        if let Some(active_control_flag) = self.active_control_flag.clone() {
            active_control_flag.set(DownloadThreadControlFlag::Stop);
        }
    }

    fn manage_completed_signal(&mut self, game_id: String) {
        info!("Got signal 'Completed'");
        if let Some(interface) = &self.current_game_interface {
            // When if let chains are stabilised, combine these two statements
            if interface.id == game_id {
                info!("Popping consumed data");
                self.download_queue.lock().unwrap().pop_front();
                self.download_agent_registry.remove(&game_id);
                self.active_control_flag = None;
                *self.progress.lock().unwrap() = None;
            }
        }
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }

    fn manage_queue_signal(&mut self, id: String, version: String, target_download_dir: usize) {
        info!("Got signal Queue");
        let download_agent = Arc::new(GameDownloadAgent::new(
            id.clone(),
            version,
            target_download_dir,
        ));
        let agent_status = GameDownloadStatus::Uninitialised;
        let interface_data = Arc::new(AgentInterfaceData {
            id,
            status: Mutex::new(agent_status),
        });
        self.download_agent_registry
            .insert(interface_data.id.clone(), download_agent);
        self.download_queue
            .lock()
            .unwrap()
            .push_back(interface_data);
    }

    fn manage_go_signal(&mut self) {
        info!("Got signal 'Go'");
        if self.active_control_flag.is_none() && !self.download_agent_registry.is_empty() {
            info!("Starting download agent");
            let download_agent = {
                let lock = self.download_queue.lock().unwrap();
                self.download_agent_registry
                    .get(&lock.front().unwrap().id)
                    .unwrap()
                    .clone()
            };
            let download_agent_interface =
                Arc::new(AgentInterfaceData::from(download_agent.clone()));
            self.current_game_interface = Some(download_agent_interface);

            let progress_object = download_agent.progress.clone();
            *self.progress.lock().unwrap() = Some(progress_object);

            let active_control_flag = download_agent.control_flag.clone();
            self.active_control_flag = Some(active_control_flag.clone());

            let sender = self.sender.clone();

            info!("Spawning download");
            spawn(move || {
                let signal = match download_agent.download() {
                    Ok(_) => DownloadManagerSignal::Completed(download_agent.id.clone()),
                    Err(e) => DownloadManagerSignal::Error(e),
                };
                sender.send(signal).unwrap();
            });
            info!("Finished spawning Download");

            active_control_flag.set(DownloadThreadControlFlag::Go);
            self.set_status(DownloadManagerStatus::Downloading);
        } else if let Some(active_control_flag) = self.active_control_flag.clone() {
            info!("Restarting current download");
            active_control_flag.set(DownloadThreadControlFlag::Go);
        } else {
            info!("Nothing was set");
        }
    }
    fn manage_error_signal(&self, error: GameDownloadError) {
        let current_status = self.current_game_interface.clone().unwrap();
        let mut lock = current_status.status.lock().unwrap();
        *lock = GameDownloadStatus::Error(error.clone());
        self.set_status(DownloadManagerStatus::Error(error));
    }
    fn set_status(&self, status: DownloadManagerStatus) {
        *self.status.lock().unwrap() = status;
    }
}
