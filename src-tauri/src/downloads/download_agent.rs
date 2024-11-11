use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::remote::RemoteAccessError;
use crate::DB;
use log::info;
use rayon::{spawn, ThreadPool, ThreadPoolBuilder};
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::Thread;
use urlencoding::encode;

#[cfg(target_os = "linux")]
use rustix::fs::{fallocate, FallocateFlags};

use super::download_logic::download_game_chunk;
use super::download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag};
use super::progress_object::ProgressObject;

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    pub target_download_dir: usize,
    contexts: Mutex<Vec<DropDownloadContext>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: ProgressObject,
}

#[derive(Debug)]
pub enum GameDownloadError {
    CommunicationError(RemoteAccessError),
    ChecksumError,
    SetupError(String),
    LockError,
}

impl Display for GameDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDownloadError::CommunicationError(error) => write!(f, "{}", error),
            GameDownloadError::SetupError(error) => write!(f, "{}", error),
            GameDownloadError::LockError => write!(f, "Failed to acquire lock. Something has gone very wrong internally. Please restart the application"),
            GameDownloadError::ChecksumError => write!(f, "Checksum failed to validate for download"),
        }
    }
}

impl GameDownloadAgent {
    pub fn new(id: String, version: String, target_download_dir: usize) -> Self {
        // Don't run by default
        let status = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);
        Self {
            id,
            version,
            control_flag: status.clone(),
            manifest: Mutex::new(None),
            target_download_dir,
            contexts: Mutex::new(Vec::new()),
            progress: ProgressObject::new(0, 0),
        }
    }

    // Blocking
    // Requires mutable self
    pub fn setup_download(&mut self) -> Result<(), GameDownloadError> {
        self.ensure_manifest_exists()?;
        info!("Ensured manifest exists");

        self.generate_contexts()?;
        info!("Generated contexts");

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub fn download(&mut self) -> Result<(), GameDownloadError> {
        self.setup_download()?;
        self.run();

        Ok(())
    }

    pub fn ensure_manifest_exists(&mut self) -> Result<(), GameDownloadError> {
        if self.manifest.lock().unwrap().is_some() {
            return Ok(());
        }

        // Explicitly propagate error
        self.download_manifest()
    }

    fn download_manifest(&mut self) -> Result<(), GameDownloadError> {
        let base_url = DB.fetch_base_url();
        let manifest_url = base_url
            .join(
                format!(
                    "/api/v1/client/metadata/manifest?id={}&version={}",
                    self.id,
                    encode(&self.version)
                )
                .as_str(),
            )
            .unwrap();

        let header = generate_authorization_header();
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(manifest_url.to_string())
            .header("Authorization", header)
            .send()
            .unwrap();

        if response.status() != 200 {
            return Err(GameDownloadError::CommunicationError(
                format!(
                    "Failed to download game manifest: {} {}",
                    response.status(),
                    response.text().unwrap()
                )
                .into(),
            ));
        }

        let manifest_download = response.json::<DropManifest>().unwrap();
        let length = manifest_download
            .values()
            .map(|chunk| {
                return chunk.lengths.iter().sum::<usize>();
            })
            .sum::<usize>();
        let chunk_count = manifest_download
            .values()
            .map(|chunk| chunk.lengths.len())
            .sum();
        self.progress = ProgressObject::new(length.try_into().unwrap(), chunk_count);

        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download);
            return Ok(());
        }

        Err(GameDownloadError::LockError)
    }

    pub fn generate_contexts(&self) -> Result<(), GameDownloadError> {
        let db_lock = DB.borrow_data().unwrap();
        let data_base_dir = db_lock.games.install_dirs[self.target_download_dir].clone();
        drop(db_lock);

        let manifest = self.manifest.lock().unwrap().clone().unwrap();
        let version = self.version.clone();
        let game_id = self.id.clone();

        let data_base_dir_path = Path::new(&data_base_dir);

        let mut contexts = Vec::new();
        let base_path = data_base_dir_path.join(game_id.clone()).clone();
        create_dir_all(base_path.clone()).unwrap();

        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(&raw_path));

            let container = path.parent().unwrap();
            create_dir_all(container).unwrap();

            let file = File::create(path.clone()).unwrap();
            let mut running_offset = 0;

            for (i, length) in chunk.lengths.iter().enumerate() {
                contexts.push(DropDownloadContext {
                    file_name: raw_path.to_string(),
                    version: version.to_string(),
                    offset: running_offset,
                    index: i,
                    game_id: game_id.to_string(),
                    path: path.clone(),
                    checksum: chunk.checksums[i].clone(),
                });
                running_offset += *length as u64;
            }

            #[cfg(target_os = "linux")]
            if running_offset > 0 {
                fallocate(file, FallocateFlags::empty(), 0, running_offset).unwrap();
            }
        }

        if let Ok(mut context_lock) = self.contexts.lock() {
            *context_lock = contexts;
            return Ok(());
        }

        Err(GameDownloadError::SetupError(
            "Failed to generate download contexts".to_owned(),
        ))
    }

    pub fn run(&self) {
        const DOWNLOAD_MAX_THREADS: usize = 4;

        let pool = ThreadPoolBuilder::new()
            .num_threads(DOWNLOAD_MAX_THREADS)
            .build()
            .unwrap();


        pool.scope(move |scope| {
            let contexts = self.contexts.lock().unwrap();

            for (index, context) in contexts.iter().enumerate() {
                let context = context.clone();
                let control_flag = self.control_flag.clone(); // Clone arcs
                let progress = self.progress.get(index); // Clone arcs

                scope.spawn(move |_| {
                    info!(
                        "starting download for file {} {}",
                        context.file_name, context.index
                    );
                    download_game_chunk(context, control_flag, progress).unwrap();
                });
            }
        });
    }
}
