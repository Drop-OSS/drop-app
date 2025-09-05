use std::collections::HashMap;
use tauri::State;
use std::sync::Mutex;

use crate::AppState;
use super::manager::PlaytimeStats;
use super::events::{push_playtime_update, push_session_start, push_session_end};

#[tauri::command]
pub fn start_playtime_tracking(
    game_id: String,
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<(), String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    match playtime_manager_lock.start_session(game_id.clone()) {
        Ok(()) => {
            push_session_start(&state_lock.app_handle, &game_id);
            Ok(())
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
pub fn end_playtime_tracking(
    game_id: String,
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<PlaytimeStats, String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    match playtime_manager_lock.end_session(game_id.clone()) {
        Ok(stats) => {
            push_session_end(&state_lock.app_handle, &game_id, &stats);
            push_playtime_update(&state_lock.app_handle, &game_id, stats.clone(), false);
            Ok(stats)
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
pub fn fetch_game_playtime(
    game_id: String,
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<Option<PlaytimeStats>, String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    Ok(playtime_manager_lock.get_game_stats(&game_id))
}

#[tauri::command]
pub fn fetch_all_playtime_stats(
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<HashMap<String, PlaytimeStats>, String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    Ok(playtime_manager_lock.get_all_stats())
}

#[tauri::command]
pub fn is_playtime_session_active(
    game_id: String,
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<bool, String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    Ok(playtime_manager_lock.is_session_active(&game_id))
}

#[tauri::command]
pub fn get_active_playtime_sessions(
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<String>, String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    Ok(playtime_manager_lock.get_active_sessions())
}

#[tauri::command]
pub fn cleanup_orphaned_playtime_sessions(
    state: State<'_, Mutex<AppState<'_>>>,
) -> Result<(), String> {
    let state_lock = state.lock().unwrap();
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    
    playtime_manager_lock.cleanup_orphaned_sessions()
        .map_err(|e| e.to_string())
}
