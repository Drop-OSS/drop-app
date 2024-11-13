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
    download_agent::GameDownloadAgent,
    download_manager_interface::DownloadManagerInterface,
    download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
    progress_object::ProgressObject,
};

pub struct DownloadManager {
    download_agent_registry: HashMap<String, Arc<GameDownloadAgent>>,
    download_queue: Arc<Mutex<VecDeque<String>>>,
    receiver: Receiver<DownloadManagerSignal>,
    sender: Sender<DownloadManagerSignal>,
    progress: Arc<Mutex<Option<ProgressObject>>>,

    current_game_id: Option<String>, // Should be the only game download agent in the map with the "Go" flag
    active_control_flag: Option<DownloadThreadControl>,
}
pub enum DownloadManagerSignal {
    Go,
    Stop,
    Completed(String),
    Queue(String, String, usize),
    Finish,
}

impl DownloadManager {
    pub fn generate() -> DownloadManagerInterface {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let (sender, receiver) = channel();
        let active_progress = Arc::new(Mutex::new(None));

        let manager = Self {
            download_agent_registry: HashMap::new(),
            download_queue: queue.clone(),
            receiver,
            current_game_id: None,
            active_control_flag: None,
            sender: sender.clone(),
            progress: active_progress.clone(),
        };

        let terminator = spawn(|| manager.manage_queue());

        DownloadManagerInterface::new(terminator, queue, active_progress, sender)
    }

    fn manage_queue(mut self) -> Result<(), ()> {
        loop {
            let signal = match self.receiver.recv() {
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
                    match self.active_control_flag {
                        Some(active_control_flag) => {
                            active_control_flag.set(DownloadThreadControlFlag::Stop)
                        }
                        None => {}
                    }
                    return Ok(());
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
        if self.current_game_id == Some(game_id.clone()) {
            info!("Popping consumed data");
            self.download_queue.lock().unwrap().pop_front();
            self.download_agent_registry.remove(&game_id);
            self.active_control_flag = None;
            *self.progress.lock().unwrap() = None;
        }
        self.sender.send(DownloadManagerSignal::Go).unwrap();
    }

    fn manage_queue_signal(
        &mut self,
        game_id: String,
        version: String,
        target_download_dir: usize,
    ) {
        info!("Got signal Queue");
        let download_agent = Arc::new(GameDownloadAgent::new(
            game_id.clone(),
            version,
            target_download_dir,
        ));
        self.download_agent_registry
            .insert(game_id.clone(), download_agent);
        self.download_queue.lock().unwrap().push_back(game_id);
    }

    fn manage_go_signal(&mut self) {
        info!("Got signal 'Go'");
        if self.active_control_flag.is_none() && !self.download_agent_registry.is_empty() {
            info!("Starting download agent");
            let download_agent = {
                let lock = self.download_queue.lock().unwrap();
                self.download_agent_registry
                    .get(&lock.front().unwrap().clone())
                    .unwrap()
                    .clone()
            };
            self.current_game_id = Some(download_agent.id.clone());

            let progress_object = download_agent.progress.clone();
            *self.progress.lock().unwrap() = Some(progress_object);

            let active_control_flag = download_agent.control_flag.clone();
            self.active_control_flag = Some(active_control_flag.clone());

            let sender = self.sender.clone();

            info!("Spawning download");
            spawn(move || {
                download_agent.download().unwrap();
                sender
                    .send(DownloadManagerSignal::Completed(download_agent.id.clone()))
                    .unwrap();
            });
            info!("Finished spawning Download");

            active_control_flag.set(DownloadThreadControlFlag::Go);
        } else if let Some(active_control_flag) = self.active_control_flag.clone() {
            info!("Restarting current download");
            active_control_flag.set(DownloadThreadControlFlag::Go);
        } else {
            info!("Nothing was set");
        }
    }
}
