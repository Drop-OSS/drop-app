use log::{debug, error};
use tauri::AppHandle;

use crate::DropFunctionState;

#[tauri::command]
pub async fn quit<>(app: tauri::AppHandle, state: tauri::State<'_, DropFunctionState<'_>>) -> Result<(), ()> {
    cleanup_and_exit(&app, &state).await;

    Ok(())
}

pub async fn cleanup_and_exit(
    app: &AppHandle,
    state: &tauri::State<'_, DropFunctionState<'_>>,
) {
    debug!("cleaning up and exiting application");
    let download_manager = state.lock().await.download_manager.clone();
    match download_manager.ensure_terminated().await {
        Ok(res) => match res {
            Ok(_) => debug!("download manager terminated correctly"),
            Err(_) => error!("download manager failed to terminate correctly"),
        },
        Err(e) => panic!("{e:?}"),
    }

    app.exit(0);
}
