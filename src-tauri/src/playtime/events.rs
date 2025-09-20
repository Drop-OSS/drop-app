use serde::Serialize;
use tauri::{AppHandle, Emitter};
use log::warn;

use super::manager::PlaytimeStats;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeUpdateEvent {
    pub game_id: String,
    pub stats: PlaytimeStats,
    pub is_active: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeSessionStartEvent {
    pub game_id: String,
    pub start_time: std::time::SystemTime,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeSessionEndEvent {
    pub game_id: String,
    pub session_duration_seconds: u64,
    pub total_playtime_seconds: u64,
    pub session_count: u32,
}

/// Push a playtime update event to the frontend
pub fn push_playtime_update(app_handle: &AppHandle, game_id: &str, stats: PlaytimeStats, is_active: bool) {
    let event = PlaytimeUpdateEvent {
        game_id: game_id.to_string(),
        stats,
        is_active,
    };

    if let Err(e) = app_handle.emit(&format!("playtime_update/{}", game_id), &event) {
        warn!("Failed to emit playtime update event for {}: {}", game_id, e);
    }

    // Also emit a general playtime update event for global listeners
    if let Err(e) = app_handle.emit("playtime_update", &event) {
        warn!("Failed to emit general playtime update event: {}", e);
    }
}

/// Push a session start event to the frontend
pub fn push_session_start(app_handle: &AppHandle, game_id: &str) {
    let event = PlaytimeSessionStartEvent {
        game_id: game_id.to_string(),
        start_time: std::time::SystemTime::now(),
    };

    if let Err(e) = app_handle.emit(&format!("playtime_session_start/{}", game_id), &event) {
        warn!("Failed to emit session start event for {}: {}", game_id, e);
    }

    if let Err(e) = app_handle.emit("playtime_session_start", &event) {
        warn!("Failed to emit general session start event: {}", e);
    }
}

/// Push a session end event to the frontend
pub fn push_session_end(app_handle: &AppHandle, game_id: &str, stats: &PlaytimeStats) {
    let event = PlaytimeSessionEndEvent {
        game_id: game_id.to_string(),
        session_duration_seconds: stats.current_session_duration.unwrap_or(0),
        total_playtime_seconds: stats.total_playtime_seconds,
        session_count: stats.session_count,
    };

    if let Err(e) = app_handle.emit(&format!("playtime_session_end/{}", game_id), &event) {
        warn!("Failed to emit session end event for {}: {}", game_id, e);
    }

    if let Err(e) = app_handle.emit("playtime_session_end", &event) {
        warn!("Failed to emit general session end event: {}", e);
    }
}
