use std::collections::HashMap;
use std::time::SystemTime;
use std::fmt;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use chrono;

use crate::database::db::{borrow_db_checked, borrow_db_mut_checked};
use crate::database::models::data::{GamePlaytimeStats, PlaytimeSession};
use crate::error::process_error::ProcessError;

#[derive(Debug)]
pub enum PlaytimeError {
    DatabaseError(String),
    SessionNotFound(String),
    SessionAlreadyActive(String),
    InvalidGameId(String),
}

impl fmt::Display for PlaytimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaytimeError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            PlaytimeError::SessionNotFound(game_id) => write!(f, "Session not found for game: {}", game_id),
            PlaytimeError::SessionAlreadyActive(game_id) => write!(f, "Session already active for game: {}", game_id),
            PlaytimeError::InvalidGameId(game_id) => write!(f, "Invalid game ID: {}", game_id),
        }
    }
}

impl std::error::Error for PlaytimeError {}

impl From<PlaytimeError> for ProcessError {
    fn from(error: PlaytimeError) -> Self {
        ProcessError::PlaytimeError(error.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeStats {
    pub game_id: String,
    pub total_playtime_seconds: u64,
    pub session_count: u32,
    #[serde(serialize_with = "serialize_system_time")]
    pub first_played: SystemTime,
    #[serde(serialize_with = "serialize_system_time")]
    pub last_played: SystemTime,
    pub average_session_length: u64,
    pub current_session_duration: Option<u64>,
}

fn serialize_system_time<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use std::time::UNIX_EPOCH;
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp_ms = duration.as_millis() as u64;
            // Convert to JavaScript-compatible ISO 8601 string
            let datetime = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            serializer.serialize_str(&datetime.to_rfc3339())
        }
        Err(_) => {
            // Fallback for times before UNIX_EPOCH
            let datetime = chrono::DateTime::from_timestamp(0, 0).unwrap();
            serializer.serialize_str(&datetime.to_rfc3339())
        }
    }
}

impl From<GamePlaytimeStats> for PlaytimeStats {
    fn from(stats: GamePlaytimeStats) -> Self {
        let average_length = stats.average_session_length();
        Self {
            game_id: stats.game_id,
            total_playtime_seconds: stats.total_playtime_seconds,
            session_count: stats.session_count,
            first_played: stats.first_played,
            last_played: stats.last_played,
            average_session_length: average_length,
            current_session_duration: None,
        }
    }
}

pub struct PlaytimeManager {
    app_handle: AppHandle,
}

impl PlaytimeManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Start tracking playtime for a game
    pub fn start_session(&self, game_id: String) -> Result<(), PlaytimeError> {
        debug!("Starting playtime session for game: {}", game_id);

        let mut db_handle = borrow_db_mut_checked();
        
        // Check if session is already active
        if db_handle.playtime_data.active_sessions.contains_key(&game_id) {
            warn!("Session already active for game: {}", game_id);
            return Err(PlaytimeError::SessionAlreadyActive(game_id));
        }

        // Create new session
        let session = PlaytimeSession::new(game_id.clone());
        db_handle.playtime_data.active_sessions.insert(game_id.clone(), session);

        debug!("Started playtime tracking for game: {}", game_id);
        Ok(())
    }

    /// End tracking playtime for a game and update stats
    pub fn end_session(&self, game_id: String) -> Result<PlaytimeStats, PlaytimeError> {
        debug!("Ending playtime session for game: {}", game_id);

        let mut db_handle = borrow_db_mut_checked();
        
        // Get active session
        let session = db_handle.playtime_data.active_sessions.remove(&game_id)
            .ok_or_else(|| PlaytimeError::SessionNotFound(game_id.clone()))?;

        let session_duration = session.duration().as_secs();
        debug!("Session duration for {}: {} seconds", game_id, session_duration);

        // Update or create game stats
        let stats = db_handle.playtime_data.game_sessions
            .entry(game_id.clone())
            .or_insert_with(|| GamePlaytimeStats::new(game_id.clone()));

        // Update stats
        stats.total_playtime_seconds += session_duration;
        stats.session_count += 1;
        stats.last_played = SystemTime::now();

        // If this is the first session, update first_played
        if stats.session_count == 1 {
            stats.first_played = session.start_time;
        }

        let result_stats = PlaytimeStats {
            game_id: stats.game_id.clone(),
            total_playtime_seconds: stats.total_playtime_seconds,
            session_count: stats.session_count,
            first_played: stats.first_played,
            last_played: stats.last_played,
            average_session_length: stats.average_session_length(),
            current_session_duration: Some(session_duration),
        };

        debug!("Updated playtime stats for {}: {} total seconds, {} sessions", 
               game_id, stats.total_playtime_seconds, stats.session_count);

        Ok(result_stats)
    }

    /// Get playtime stats for a specific game
    pub fn get_game_stats(&self, game_id: &str) -> Option<PlaytimeStats> {
        let db_handle = borrow_db_checked();
        
        if let Some(stats) = db_handle.playtime_data.game_sessions.get(game_id) {
            let mut playtime_stats: PlaytimeStats = stats.clone().into();
            
            // If there's an active session, include current session duration
            if let Some(session) = db_handle.playtime_data.active_sessions.get(game_id) {
                playtime_stats.current_session_duration = Some(session.duration().as_secs());
            }
            
            Some(playtime_stats)
        } else {
            None
        }
    }

    /// Get playtime stats for all games
    pub fn get_all_stats(&self) -> HashMap<String, PlaytimeStats> {
        let db_handle = borrow_db_checked();
        let mut result = HashMap::new();

        for (game_id, stats) in &db_handle.playtime_data.game_sessions {
            let mut playtime_stats: PlaytimeStats = stats.clone().into();
            
            // If there's an active session, include current session duration
            if let Some(session) = db_handle.playtime_data.active_sessions.get(game_id) {
                playtime_stats.current_session_duration = Some(session.duration().as_secs());
            }
            
            result.insert(game_id.clone(), playtime_stats);
        }

        result
    }

    /// Check if a game has an active session
    pub fn is_session_active(&self, game_id: &str) -> bool {
        let db_handle = borrow_db_checked();
        db_handle.playtime_data.active_sessions.contains_key(game_id)
    }

    /// Get active sessions (for debugging/monitoring)
    pub fn get_active_sessions(&self) -> Vec<String> {
        let db_handle = borrow_db_checked();
        db_handle.playtime_data.active_sessions.keys().cloned().collect()
    }

    /// Clean up any orphaned sessions (called on startup)
    pub fn cleanup_orphaned_sessions(&self) -> Result<(), PlaytimeError> {
        debug!("Cleaning up orphaned playtime sessions");
        
        let mut db_handle = borrow_db_mut_checked();
        let orphaned_sessions: Vec<String> = db_handle.playtime_data.active_sessions.keys().cloned().collect();
        
        for game_id in orphaned_sessions {
            warn!("Found orphaned session for game: {}, ending it", game_id);
            
            if let Some(session) = db_handle.playtime_data.active_sessions.remove(&game_id) {
                let session_duration = session.duration().as_secs();
                
                // Only count sessions that lasted more than 5 seconds to avoid counting crashes
                if session_duration > 5 {
                    let stats = db_handle.playtime_data.game_sessions
                        .entry(game_id.clone())
                        .or_insert_with(|| GamePlaytimeStats::new(game_id.clone()));

                    stats.total_playtime_seconds += session_duration;
                    stats.session_count += 1;
                    stats.last_played = SystemTime::now();

                    if stats.session_count == 1 {
                        stats.first_played = session.start_time;
                    }

                    debug!("Recovered orphaned session for {}: {} seconds", game_id, session_duration);
                } else {
                    debug!("Discarded short orphaned session for {}: {} seconds", game_id, session_duration);
                }
            }
        }

        Ok(())
    }

    // Future server-side methods (ready for migration)
    
    /// Start session with server sync (placeholder for future implementation)
    #[allow(dead_code)]
    pub async fn sync_session_start(&self, game_id: String) -> Result<(), PlaytimeError> {
        // For now, just call local method
        self.start_session(game_id)?;
        
        // Future: Send to server
        // let response = self.api_client.post("/api/v1/playtime/start")
        //     .json(&StartSessionRequest { game_id })
        //     .send().await?;
        
        Ok(())
    }

    /// End session with server sync (placeholder for future implementation)
    #[allow(dead_code)]
    pub async fn sync_session_end(&self, game_id: String) -> Result<PlaytimeStats, PlaytimeError> {
        // For now, just call local method
        let stats = self.end_session(game_id)?;
        
        // Future: Send to server
        // let response = self.api_client.post("/api/v1/playtime/end")
        //     .json(&EndSessionRequest { game_id, duration: stats.current_session_duration })
        //     .send().await?;
        
        Ok(stats)
    }
}
