use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
    usize,
};

use log::{debug, error};
use reqwest::redirect::Policy;
use tauri::{AppHandle, Emitter};

use crate::{
    database::
        models::data::{DownloadType, DownloadableMetadata}
    ,
    download_manager::{
        download_manager::{DownloadManagerSignal, DownloadStatus},
        downloadable::Downloadable,
        util::{
            download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
            progress_object::{ProgressHandle, ProgressObject},
        },
    },
    error::application_download_error::ApplicationDownloadError,
    games::downloads::download_logic::{DropDownloadPipeline, DropWriter},
    DB,
};

pub struct URLDownloader {
    id: String,
    version: String,
    url: String,
    control_flag: DownloadThreadControl,
    progress: Arc<ProgressObject>,
    target: PathBuf,
    sender: Sender<DownloadManagerSignal>,
    status: Mutex<DownloadStatus>,
}

struct URLDownloaderManager {
    current_offset: usize,
}

impl URLDownloader {
    pub fn new<S: Into<String>, P: AsRef<Path>>(
        id: String,
        target: P,
        sender: Sender<DownloadManagerSignal>,
        url: S,
    ) -> Self {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        Self {
            id,
            version: String::new(),
            control_flag,
            target: target.as_ref().into(),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            status: Mutex::new(DownloadStatus::Queued),
            url: url.into(),
        }
    }

    fn download(&self, _app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        // TODO: Fix these unwraps and implement From<io::Error> for ApplicationDownloadError
        let client = reqwest::blocking::Client::builder()
            .redirect(Policy::default())
            .build()
            .unwrap();

        let response = client.head(&self.url).send().unwrap();
        let content_length = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .map(|x| x.to_str().unwrap().parse().unwrap())
            .unwrap_or(usize::MAX);
        let response = client.get(&self.url).send().unwrap();

        self.set_progress_object_params(content_length);

        let progress = self.progress.get(0);

        let progress_handle = ProgressHandle::new(progress, self.progress.clone());

        let mut pipeline = DropDownloadPipeline::new(
            response,
            DropWriter::new(&self.target),
            &self.control_flag,
            progress_handle,
            content_length,
        );
        

        let completed = pipeline
            .copy()
            .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
        if !completed {
            return Ok(false);
        };

        Ok(true)
    }
    fn set_progress_object_params(&self, max: usize) {
        // Avoid re-setting it
        if self.progress.get_max() != 0 {
            return;
        }

        self.progress.set_max(max);
        self.progress.set_size(1);
        self.progress.set_time_now();
    }
}

impl Downloadable for URLDownloader {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *self.status.lock().unwrap() = DownloadStatus::Downloading;
        self.download(app_handle)
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
            download_type: DownloadType::Tool,
        }
    }

    fn on_initialised(&self, _app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
    }

    fn on_error(&self, app_handle: &tauri::AppHandle, error: &ApplicationDownloadError) {
        *self.status.lock().unwrap() = DownloadStatus::Error;
        app_handle
            .emit("download_error", error.to_string())
            .unwrap();

        error!("error while managing download: {}", error);

        let mut handle = DB.borrow_data_mut().unwrap();
        handle
            .applications
            .transient_statuses
            .remove(&self.metadata());
    }

    fn on_complete(&self, _app_handle: &tauri::AppHandle) {
        debug!("Completed url download");
    }

    // TODO: fix this function. It doesn't restart the download properly, nor does it reset the state properly
    fn on_incomplete(&self, _app_handle: &tauri::AppHandle) {
        debug!("Incomplete url download");
    }

    fn on_cancelled(&self, _app_handle: &tauri::AppHandle) {
        debug!("Cancelled url download");
    }

    fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }
}
