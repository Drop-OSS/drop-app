use database::{
    ApplicationTransientStatus, DownloadType, DownloadableMetadata, borrow_db_checked,
    borrow_db_mut_checked,
};
use download_manager::download_manager_frontend::{DownloadManagerSignal, DownloadStatus};
use download_manager::downloadable::Downloadable;
use download_manager::error::ApplicationDownloadError;
use download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use download_manager::util::progress_object::{ProgressHandle, ProgressObject};
use log::{debug, error, info, warn};
use rayon::ThreadPoolBuilder;
use remote::auth::generate_authorization_header;
use remote::error::RemoteAccessError;
use remote::requests::generate_url;
use remote::utils::{DROP_CLIENT_ASYNC, DROP_CLIENT_SYNC};
use std::collections::{HashMap, HashSet};
use std::fs::{OpenOptions, create_dir_all};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::AppHandle;
use utils::{app_emit, lock, send};

#[cfg(target_os = "linux")]
use rustix::fs::{FallocateFlags, fallocate};

use crate::downloads::manifest::{
    DownloadBucket, DownloadContext, DownloadDrop, DropManifest, DropValidateContext, ManifestBody,
};
use crate::downloads::utils::get_disk_available;
use crate::downloads::validate::validate_game_chunk;
use crate::library::{on_game_complete, push_game_update, set_partially_installed};
use crate::state::GameStatusManager;

use super::download_logic::download_game_bucket;
use super::drop_data::DropData;

static RETRY_COUNT: usize = 3;

const TARGET_BUCKET_SIZE: usize = 63 * 1000 * 1000;
const MAX_FILES_PER_BUCKET: usize = (1024 / 4) - 1;

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    buckets: Mutex<Vec<DownloadBucket>>,
    context_map: Mutex<HashMap<String, bool>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub dropdata: DropData,
    status: Mutex<DownloadStatus>,
}

impl GameDownloadAgent {
    pub async fn new_from_index(
        id: String,
        version: String,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>,
    ) -> Result<Self, ApplicationDownloadError> {
        let base_dir = {
            let db_lock = borrow_db_checked();

            db_lock.applications.install_dirs[target_download_dir].clone()
        };

        Self::new(id, version, base_dir, sender).await
    }
    pub async fn new(
        id: String,
        version: String,
        base_dir: PathBuf,
        sender: Sender<DownloadManagerSignal>,
    ) -> Result<Self, ApplicationDownloadError> {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        let base_dir_path = Path::new(&base_dir);
        let data_base_dir_path = base_dir_path.join(id.clone());

        let stored_manifest =
            DropData::generate(id.clone(), version.clone(), data_base_dir_path.clone());

        let context_lock = stored_manifest.contexts.lock().unwrap().clone();

        let result = Self {
            id,
            version,
            control_flag,
            manifest: Mutex::new(None),
            buckets: Mutex::new(Vec::new()),
            context_map: Mutex::new(HashMap::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            dropdata: stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
        };

        result.ensure_manifest_exists().await?;

        let required_space = lock!(result.manifest)
            .as_ref()
            .unwrap()
            .values()
            .map(|e| {
                e.lengths
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *context_lock.get(&e.checksums[*i]).unwrap_or(&false))
                    .map(|(_, v)| v)
                    .sum::<usize>()
            })
            .sum::<usize>() as u64;

        let available_space = get_disk_available(data_base_dir_path)? as u64;

        if required_space > available_space {
            return Err(ApplicationDownloadError::DiskFull(
                required_space,
                available_space,
            ));
        }

        Ok(result)
    }

    // Blocking
    pub fn setup_download(&self, app_handle: &AppHandle) -> Result<(), ApplicationDownloadError> {
        let mut db_lock = borrow_db_mut_checked();
        let status = ApplicationTransientStatus::Downloading {
            version_name: self.version.clone(),
        };
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        // Don't use GameStatusManager because this game isn't installed
        push_game_update(app_handle, &self.metadata().id, None, (None, Some(status)));

        if !self.check_manifest_exists() {
            return Err(ApplicationDownloadError::NotInitialized);
        }

        self.ensure_buckets()?;

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_download(app_handle)?;
        let timer = Instant::now();

        info!("beginning download for {}...", self.metadata().id);

        let res = self.run().map_err(ApplicationDownloadError::Communication);

        debug!(
            "{} took {}ms to download",
            self.id,
            timer.elapsed().as_millis()
        );
        res
    }

    pub fn check_manifest_exists(&self) -> bool {
        lock!(self.manifest).is_some()
    }

    pub async fn ensure_manifest_exists(&self) -> Result<(), ApplicationDownloadError> {
        if lock!(self.manifest).is_some() {
            return Ok(());
        }

        self.download_manifest().await
    }

    async fn download_manifest(&self) -> Result<(), ApplicationDownloadError> {
        let client = DROP_CLIENT_ASYNC.clone();
        let url = generate_url(
            &["/api/v1/client/game/manifest"],
            &[("id", &self.id), ("version", &self.version)],
        )
        .map_err(ApplicationDownloadError::Communication)?;

        let response = client
            .get(url)
            .header("Authorization", generate_authorization_header())
            .send()
            .await
            .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

        if response.status() != 200 {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::ManifestDownloadFailed(
                    response.status(),
                    response.text().await.unwrap(),
                ),
            ));
        }

        let manifest_download: DropManifest = response
            .json()
            .await
            .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download);
            return Ok(());
        }

        Err(ApplicationDownloadError::Lock)
    }

    // Sets it up for both download and validate
    fn setup_progress(&self) {
        let buckets = lock!(self.buckets);

        let chunk_count = buckets.iter().map(|e| e.drops.len()).sum();

        let total_length = buckets
            .iter()
            .map(|bucket| bucket.drops.iter().map(|e| e.length).sum::<usize>())
            .sum();

        self.progress.set_max(total_length);
        self.progress.set_size(chunk_count);
        self.progress.reset();
    }

    pub fn ensure_buckets(&self) -> Result<(), ApplicationDownloadError> {
        if lock!(self.buckets).is_empty() {
            self.generate_buckets()?;
        }

        *lock!(self.context_map) = self.dropdata.get_contexts();

        Ok(())
    }

    pub fn generate_buckets(&self) -> Result<(), ApplicationDownloadError> {
        let manifest = lock!(self.manifest)
            .clone()
            .ok_or(ApplicationDownloadError::NotInitialized)?;
        let game_id = self.id.clone();

        let base_path = Path::new(&self.dropdata.base_path);
        create_dir_all(base_path)?;

        let mut buckets = Vec::new();

        let mut current_buckets = HashMap::<String, DownloadBucket>::new();
        let mut current_bucket_sizes = HashMap::<String, usize>::new();

        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(&raw_path));

            let container = path
                .parent()
                .ok_or(ApplicationDownloadError::IoError(Arc::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    "no parent directory",
                ))))?;
            create_dir_all(container)?;

            let already_exists = path.exists();
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)?;
            let mut file_running_offset = 0;

            for (index, length) in chunk.lengths.iter().enumerate() {
                let drop = DownloadDrop {
                    filename: raw_path.to_string(),
                    start: file_running_offset,
                    length: *length,
                    checksum: chunk.checksums[index].clone(),
                    permissions: chunk.permissions,
                    path: path.clone(),
                    index,
                };
                file_running_offset += *length;

                if *length >= TARGET_BUCKET_SIZE {
                    // They get their own bucket

                    buckets.push(DownloadBucket {
                        game_id: game_id.clone(),
                        version: chunk.version_name.clone(),
                        drops: vec![drop],
                    });

                    continue;
                }

                let current_bucket_size = current_bucket_sizes
                    .entry(chunk.version_name.clone())
                    .or_insert_with(|| 0);
                let c_version_name = chunk.version_name.clone();
                let c_game_id = game_id.clone();
                let current_bucket = current_buckets
                    .entry(chunk.version_name.clone())
                    .or_insert_with(|| DownloadBucket {
                        game_id: c_game_id,
                        version: c_version_name,
                        drops: vec![],
                    });

                if (*current_bucket_size + length >= TARGET_BUCKET_SIZE
                    || current_bucket.drops.len() >= MAX_FILES_PER_BUCKET)
                    && !current_bucket.drops.is_empty()
                {
                    // Move current bucket into list and make a new one
                    buckets.push(current_bucket.clone());
                    *current_bucket = DownloadBucket {
                        game_id: game_id.clone(),
                        version: chunk.version_name.clone(),
                        drops: vec![],
                    };
                    *current_bucket_size = 0;
                }

                current_bucket.drops.push(drop);
                *current_bucket_size += *length;
            }

            #[cfg(target_os = "linux")]
            if file_running_offset > 0 && !already_exists {
                let _ = fallocate(file, FallocateFlags::empty(), 0, file_running_offset as u64);
            }
        }

        for (_, bucket) in current_buckets.into_iter() {
            if !bucket.drops.is_empty() {
                buckets.push(bucket);
            }
        }

        info!("buckets: {}", buckets.len());

        let existing_contexts = self.dropdata.get_contexts();
        self.dropdata.set_contexts(
            &buckets
                .iter()
                .flat_map(|x| x.drops.iter().map(|v| v.checksum.clone()))
                .map(|x| {
                    let contains = existing_contexts.get(&x).unwrap_or(&false);
                    (x, *contains)
                })
                .collect::<Vec<(String, bool)>>(),
        );

        *lock!(self.buckets) = buckets;

        Ok(())
    }

    fn run(&self) -> Result<bool, RemoteAccessError> {
        self.setup_progress();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        debug!(
            "downloading game: {} with {} threads",
            self.id, max_download_threads
        );
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap_or_else(|_| {
                panic!("failed to build thread pool with {max_download_threads} threads")
            });

        let buckets = lock!(self.buckets);

        let mut download_contexts = HashMap::<String, DownloadContext>::new();

        let versions = buckets
            .iter()
            .map(|e| &e.version)
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>();

        info!("downloading across these versions: {versions:?}");

        let completed_contexts = Arc::new(boxcar::Vec::new());
        let completed_indexes_loop_arc = completed_contexts.clone();

        for version in versions {
            let download_context = DROP_CLIENT_SYNC
                .post(generate_url(&["/api/v2/client/context"], &[])?)
                .json(&ManifestBody {
                    game: self.id.clone(),
                    version: version.clone(),
                })
                .header("Authorization", generate_authorization_header())
                .send()?;

            if download_context.status() != 200 {
                return Err(RemoteAccessError::InvalidResponse(download_context.json()?));
            }

            let download_context = download_context.json::<DownloadContext>()?;
            info!(
                "download context: ({}) {}",
                &version, download_context.context
            );
            download_contexts.insert(version, download_context);
        }

        let download_contexts = &download_contexts;

        pool.scope(|scope| {
            let context_map = lock!(self.context_map);
            for (index, bucket) in buckets.iter().enumerate() {
                let mut bucket = (*bucket).clone();
                let completed_contexts = completed_indexes_loop_arc.clone();

                let progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());

                // If we've done this one already, skip it
                // Note to future DecDuck, DropData gets loaded into context_map
                let todo_drops = bucket
                    .drops
                    .into_iter()
                    .filter(|e| {
                        let todo = !*context_map.get(&e.checksum).unwrap_or(&false);
                        if !todo {
                            progress_handle.skip(e.length);
                        }
                        todo
                    })
                    .collect::<Vec<DownloadDrop>>();

                if todo_drops.is_empty() {
                    continue;
                };

                bucket.drops = todo_drops;

                let sender = self.sender.clone();

                let download_context =
                    download_contexts.get(&bucket.version).unwrap_or_else(|| {
                        panic!(
                            "Could not get bucket version {}. Corrupted state.",
                            bucket.version
                        )
                    });

                scope.spawn(move |_| {
                    // 3 attempts
                    for i in 0..RETRY_COUNT {
                        let loop_progress_handle = progress_handle.clone();
                        match download_game_bucket(
                            &bucket,
                            download_context,
                            &self.control_flag,
                            loop_progress_handle,
                        ) {
                            Ok(true) => {
                                for drop in bucket.drops {
                                    completed_contexts.push(drop.checksum);
                                }
                                return;
                            }
                            Ok(false) => return,
                            Err(e) => {
                                warn!("game download agent error: {e}");

                                let retry = matches!(
                                    &e,
                                    ApplicationDownloadError::Communication(_)
                                        | ApplicationDownloadError::Checksum
                                        | ApplicationDownloadError::Lock
                                        | ApplicationDownloadError::IoError(_)
                                );

                                if i == RETRY_COUNT - 1 || !retry {
                                    warn!("retry logic failed, not re-attempting.");
                                    send!(sender, DownloadManagerSignal::Error(e));
                                    return;
                                }
                            }
                        }
                    }
                });
            }
        });

        let newly_completed = completed_contexts.clone();

        let completed_lock_len = {
            let mut context_map_lock = lock!(self.context_map);
            for (_, item) in newly_completed.iter() {
                context_map_lock.insert(item.clone(), true);
            }

            context_map_lock.values().filter(|x| **x).count()
        };

        let context_map_lock = lock!(self.context_map);
        let contexts = buckets
            .iter()
            .flat_map(|x| x.drops.iter().map(|e| e.checksum.clone()))
            .map(|x| {
                let completed = context_map_lock.get(&x).unwrap_or(&false);
                (x, *completed)
            })
            .collect::<Vec<(String, bool)>>();
        drop(context_map_lock);

        self.dropdata.set_contexts(&contexts);
        self.dropdata.write();

        // If there are any contexts left which are false
        if !contexts.iter().all(|x| x.1) {
            info!(
                "download agent for {} exited without completing ({}/{}) ({} buckets)",
                self.id.clone(),
                completed_lock_len,
                contexts.len(),
                buckets.len()
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn setup_validate(&self, app_handle: &AppHandle) {
        self.setup_progress();

        self.control_flag.set(DownloadThreadControlFlag::Go);

        let status = ApplicationTransientStatus::Validating {
            version_name: self.version.clone(),
        };

        let mut db_lock = borrow_db_mut_checked();
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        push_game_update(app_handle, &self.metadata().id, None, (None, Some(status)));
    }

    pub fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_validate(app_handle);

        let buckets = lock!(self.buckets);
        let contexts: Vec<DropValidateContext> = buckets
            .clone()
            .into_iter()
            .flat_map(|e| -> Vec<DropValidateContext> { e.into() })
            .collect();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        info!("{} validation contexts", contexts.len());
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap_or_else(|_| {
                panic!("failed to build thread pool with {max_download_threads} threads")
            });

        let invalid_chunks = Arc::new(boxcar::Vec::new());
        pool.scope(|scope| {
            for (index, context) in contexts.iter().enumerate() {
                let current_progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(current_progress, self.progress.clone());
                let invalid_chunks_scoped = invalid_chunks.clone();
                let sender = self.sender.clone();

                scope.spawn(move |_| {
                    match validate_game_chunk(context, &self.control_flag, progress_handle) {
                        Ok(true) => {}
                        Ok(false) => {
                            invalid_chunks_scoped.push(context.checksum.clone());
                        }
                        Err(e) => {
                            error!("{e}");
                            send!(sender, DownloadManagerSignal::Error(e));
                        }
                    }
                });
            }
        });

        // If there are any contexts left which are false
        if !invalid_chunks.is_empty() {
            info!("validation of game id {} failed", self.id);

            for context in invalid_chunks.iter() {
                self.dropdata.set_context(context.1.clone(), false);
            }

            self.dropdata.write();

            return Ok(false);
        }

        Ok(true)
    }

    pub fn cancel(&self, app_handle: &AppHandle) {
        // See docs on usage
        set_partially_installed(
            &self.metadata(),
            self.dropdata.base_path.display().to_string(),
            Some(app_handle),
        );

        self.dropdata.write();
    }
}

impl Downloadable for GameDownloadAgent {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *lock!(self.status) = DownloadStatus::Downloading;
        self.download(app_handle)
    }

    fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *lock!(self.status) = DownloadStatus::Validating;
        self.validate(app_handle)
    }

    fn progress(&self) -> Arc<ProgressObject> {
        self.progress.clone()
    }

    fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }

    fn metadata(&self) -> DownloadableMetadata {
        DownloadableMetadata {
            id: self.id.clone(),
            version: Some(self.version.clone()),
            download_type: DownloadType::Game,
        }
    }

    fn on_queued(&self, app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
        let mut db_lock = borrow_db_mut_checked();
        let status = ApplicationTransientStatus::Queued {
            version_name: self.version.clone(),
        };
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        push_game_update(app_handle, &self.id, None, (None, Some(status)));
    }

    fn on_error(&self, app_handle: &tauri::AppHandle, error: &ApplicationDownloadError) {
        *lock!(self.status) = DownloadStatus::Error;
        app_emit!(app_handle, "download_error", error.to_string());

        error!("error while managing download: {error:?}");

        let mut handle = borrow_db_mut_checked();
        handle
            .applications
            .transient_statuses
            .remove(&self.metadata());

        push_game_update(
            app_handle,
            &self.id,
            None,
            GameStatusManager::fetch_state(&self.id, &handle),
        );
    }

    fn on_complete(&self, app_handle: &tauri::AppHandle) {
        match on_game_complete(
            &self.metadata(),
            self.dropdata.base_path.to_string_lossy().to_string(),
            app_handle,
        ) {
            Ok(_) => {}
            Err(e) => {
                error!("could not mark game as complete: {e}");
                send!(
                    self.sender,
                    DownloadManagerSignal::Error(ApplicationDownloadError::DownloadError(e))
                );
            }
        }
    }

    fn on_cancelled(&self, app_handle: &tauri::AppHandle) {
        info!("cancelled {}", self.id);
        self.cancel(app_handle);
    }

    fn status(&self) -> DownloadStatus {
        lock!(self.status).clone()
    }
}
