use std::{
    collections::{HashMap, VecDeque}, sync::{mpsc::{channel, Receiver, SendError, Sender}, Arc, Mutex, MutexGuard}, thread::{spawn, JoinHandle},
};

use log::info;

use super::{download_agent::GameDownloadAgent, download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag}, progress_object::ProgressObject};

pub struct DownloadManager {
    download_agent_registry: HashMap<String, Arc<GameDownloadAgent>>,
    download_queue: Arc<Mutex<VecDeque<String>>>,
    receiver: Receiver<DownloadManagerSignal>,
    sender: Sender<DownloadManagerSignal>,
    progress: Arc<Mutex<Option<ProgressObject>>>,

    current_game_id: Option<String>, // Should be the only game download agent in the map with the "Go" flag
    active_control_flag: Option<DownloadThreadControl>
}
pub struct DownloadManagerInterface {
    terminator: JoinHandle<Result<(),()>>,
    download_queue: Arc<Mutex<VecDeque<String>>>,
    progress: Arc<Mutex<Option<ProgressObject>>>,
    sender: Sender<DownloadManagerSignal>,
}
pub enum DownloadManagerSignal {
    Go,
    Stop,
    Completed(String),
    Queue(String, String, usize)
}

impl DownloadManagerInterface {
    pub fn queue_game(&self, game_id: String, version: String, target_download_dir: usize) -> Result<(), SendError<DownloadManagerSignal>> {
        info!("Adding game id {}", game_id);
        self.sender.send(DownloadManagerSignal::Queue(game_id, version, target_download_dir))?;
        self.sender.send(DownloadManagerSignal::Go)
    }
    pub fn edit(&self) -> MutexGuard<'_, VecDeque<String>> {
        self.download_queue.lock().unwrap()
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
    }
    pub fn rearrange(&self, current_index: usize, new_index: usize) {
        let mut queue = self.edit();
        let to_move = queue.remove(current_index).unwrap();
        queue.insert(new_index, to_move);
    }
    pub fn remove_from_queue(&self, index: usize) {
        self.edit().remove(index);
    }
    pub fn remove_from_queue_string(&self, game_id: String) {
        let mut queue = self.edit();
        let current_index = get_index_from_id(&mut queue, game_id).unwrap();
        queue.remove(current_index);
    }
    pub fn pause_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.sender.send(DownloadManagerSignal::Stop)
    }
    pub fn resume_downloads(&self) -> Result<(), SendError<DownloadManagerSignal>> {
        self.sender.send(DownloadManagerSignal::Go)
    }
    pub fn ensure_terminated(self) -> Result<(), ()> {
        match self.terminator.join() {
            Ok(o) => o,
            Err(_) => Err(()),
        }
    }
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

        let terminator = spawn(|| {manager.manage_queue()});

        let interface = DownloadManagerInterface {
            terminator,
            download_queue: queue,
            sender,
            progress: active_progress
        };
        return interface;
    }

    fn manage_queue(mut self) -> Result<(), ()> {
        loop {
            let signal = match self.receiver.recv() {
                Ok(signal) => signal,
                Err(e) => {
                    return Err(())
                },
            };

            match signal {
                DownloadManagerSignal::Go => {
                    info!("Got signal 'Go'");
                    if self.active_control_flag.is_none() && !self.download_agent_registry.is_empty() {
                        info!("Starting download agent");
                        let download_agent = {
                            let lock = self.download_queue.lock().unwrap();
                            self.download_agent_registry.get(&lock.front().unwrap().clone()).unwrap().clone()
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
                            sender.send(DownloadManagerSignal::Completed(download_agent.id.clone())).unwrap();
                        });
                        info!("Finished spawning Download");

                        active_control_flag.set(DownloadThreadControlFlag::Go);
                    }
                    else if let Some(active_control_flag) = self.active_control_flag.clone() {
                        info!("Restarting current download");
                        active_control_flag.set(DownloadThreadControlFlag::Go);
                    }
                    else {
                        info!("Nothing was set");
                    }
                },
                DownloadManagerSignal::Stop => {
                    info!("Got signal 'Stop'");
                    if let Some(active_control_flag) = self.active_control_flag.clone() {
                        active_control_flag.set(DownloadThreadControlFlag::Stop);
                    }
                },
                DownloadManagerSignal::Completed(game_id) => {
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
                DownloadManagerSignal::Queue(game_id, version, target_download_dir) => {
                    info!("Got signal Queue");
                    let download_agent = Arc::new(GameDownloadAgent::new(game_id.clone(), version, target_download_dir));
                    self.download_agent_registry.insert(game_id.clone(), download_agent);
                    self.download_queue.lock().unwrap().push_back(game_id);
                },
            };
        }
    }
}

pub fn get_index_from_id(queue: &mut MutexGuard<'_, VecDeque<String>>, id: String) -> Option<usize> {
    queue.iter().position(|download_agent| {
        download_agent == &id
    })
}
