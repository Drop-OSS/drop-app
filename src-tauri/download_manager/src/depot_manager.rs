use std::{
    collections::HashMap,
    sync::RwLock,
    time::{Duration, Instant}, usize,
};

use futures_util::StreamExt;
use log::warn;
use remote::{
    error::RemoteAccessError,
    requests::{generate_url, make_authenticated_get},
    utils::DROP_CLIENT_ASYNC,
};
use serde::Deserialize;
use tauri::Url;

use crate::util::semaphore::{SyncSemaphore, SyncSemaphorePermit};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepotManifestContent {
    version_id: String,
    //compression: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepotManifest {
    content: HashMap<String, Vec<DepotManifestContent>>,
}

struct Depot {
    endpoint: String,
    manifest: Option<DepotManifest>,
    latest_speed: Option<usize>, // bytes per second
    current_downloads: SyncSemaphore,
    enabled: bool
}

pub struct DepotManager {
    depots: RwLock<Vec<Depot>>,
}

#[derive(Deserialize)]
struct ServersideDepot {
    endpoint: String,
}

const SPEEDTEST_TIMEOUT: Duration = Duration::from_secs(4);

impl Default for DepotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DepotManager {
    pub fn new() -> Self {
        Self {
            depots: RwLock::new(Vec::new()),
        }
    }

    async fn sync_depot(&self, depot: &mut Depot) -> Result<(), RemoteAccessError> {
        let manifest_url = Url::parse(&depot.endpoint)?.join("manifest.json")?;
        let manifest = DROP_CLIENT_ASYNC.get(manifest_url).send().await?;
        let manifest: DepotManifest = manifest.json().await?;
        depot.manifest.replace(manifest);

        let speedtest_url = Url::parse(&depot.endpoint)?.join("speedtest")?;
        let speedtest = DROP_CLIENT_ASYNC.get(speedtest_url).send().await?;

        let mut stream = speedtest.bytes_stream();
        let start = Instant::now();
        let mut total_length = 0;

        while let Some(chunk) = stream.next().await {
            let length = chunk?.len();
            total_length += length;
            if SPEEDTEST_TIMEOUT <= start.elapsed() {
                break;
            }
        }

        let elapsed = start.elapsed().as_millis() as usize;
        let speed = total_length.checked_div(elapsed).unwrap_or(usize::MAX);
        depot.latest_speed.replace(speed);

        Ok(())
    }

    pub async fn sync_depots(&self) -> Result<(), RemoteAccessError> {
        let depots = make_authenticated_get(generate_url(&["/api/v1/client/depots"], &[])?).await?;
        let depots: Vec<ServersideDepot> = depots.json().await?;

        let mut new_depots = depots
            .into_iter()
            .map(|depot| Depot {
                endpoint: if depot.endpoint.ends_with("/") {
                    depot.endpoint
                } else {
                    format!("{}/", depot.endpoint)
                },
                manifest: None,
                latest_speed: None,
                current_downloads: SyncSemaphore::new(),
                enabled: true,
            })
            .collect::<Vec<Depot>>();

        for depot in &mut new_depots {
            if let Err(sync_error) = self.sync_depot(depot).await {
                warn!("failed to sync depot {}: {:?}", depot.endpoint, sync_error);
                depot.enabled = false;
            }
        }
        
        let enabled = new_depots.iter().filter(|v| v.enabled).count();
        if enabled == 0 {
            return Err(RemoteAccessError::NoDepots);
        }

        let mut depot_lock = self.depots.write().unwrap();
        *depot_lock = new_depots;

        Ok(())
    }

    pub fn next_depot(
        &self,
        game_id: &str,
        version_id: &str,
    ) -> Result<(String, SyncSemaphorePermit), RemoteAccessError> {
        let lock = self.depots.read().unwrap();
        let best_depot = lock
            .iter()
            .filter(|v| {
                let manifest = match &v.manifest {
                    Some(v) => v,
                    None => return false,
                };

                let versions = match manifest.content.get(game_id) {
                    Some(v) => v,
                    None => return false,
                };

                let _version = match versions.iter().find(|v| v.version_id == version_id) {
                    Some(v) => v,
                    None => return false,
                };

                true
            })
            .max_by(|x, y| {
                let x_speed = x.latest_speed.unwrap_or(0) / x.current_downloads.permits();
                let y_speed = y.latest_speed.unwrap_or(0) / y.current_downloads.permits();
                x_speed.cmp(&y_speed)
            })
            .ok_or(RemoteAccessError::NoDepots)?;

        Ok((
            best_depot.endpoint.clone(),
            best_depot.current_downloads.acquire(),
        ))
    }
}
