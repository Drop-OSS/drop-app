pub mod commands;

use crate::error::remote_access_error::RemoteAccessError;
use log::{debug, warn, info};
use reqwest::blocking::Client;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use std::cmp::Ordering;

const GITHUB_TAGS_URL: &str = "https://api.github.com/repos/Drop-OSS/drop-app/tags";

#[derive(Deserialize)]
struct GitHubTag {
    name: String,
}

// Helper function to split version string into components
fn parse_version(version: &str) -> Vec<String> {
    version.trim_start_matches('v')
        .split(|c| c == '.' || c == '-')
        .map(|s| s.to_string())
        .collect()
}

// ersion comparison function
fn compare_versions(a: &str, b: &str) -> Ordering {
    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    // First compare the numeric parts (0.2.0 vs 0.1.0)
    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        if let (Ok(a_num), Ok(b_num)) = (a_part.parse::<u32>(), b_part.parse::<u32>()) {
            match a_num.cmp(&b_num) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        break;
    }

    // If numeric parts are equal, shorter version (without pre-release tags) wins
    match a_parts.len().cmp(&b_parts.len()) {
        Ordering::Equal => b.cmp(a),
        other => other.reverse(),
    }
}

pub fn check_for_updates(app_handle: &AppHandle) -> Result<(), RemoteAccessError> {
    let current_version = env!("CARGO_PKG_VERSION");
    info!("Manual update check initiated. Current version: {}", current_version);

    let client = Client::new();
    debug!("Sending request to GitHub API: {}", GITHUB_TAGS_URL);
    let response = client
        .get(GITHUB_TAGS_URL)
        .header("User-Agent", "Drop-App")
        .send()?;

    if !response.status().is_success() {
        warn!("Failed to fetch tags from GitHub. Status: {}", response.status());
        return Ok(());
    }

    debug!("Successfully received response from GitHub");
    let mut tags: Vec<GitHubTag> = response.json()?;
    
    // Sort tags with the comparison
    tags.sort_by(|a, b| compare_versions(&b.name, &a.name));
    
    if let Some(latest_tag) = tags.first() {
        let latest_version = latest_tag.name.trim_start_matches('v');
        info!("Latest version from GitHub: {}", latest_version);
        
        if compare_versions(latest_version, current_version) == Ordering::Greater {
            info!("New version available. Current: {}, Latest: {}", current_version, latest_version);
            
            app_handle.emit(
                "create_modal",
                serde_json::json!({
                    "type": "notification",
                    "data": {
                        "title": "Update Available",
                        "description": format!("Version {} is now available", latest_version),
                        "buttonText": "Close"
                    }
                })
            ).unwrap();
        } else {
            info!("Application is up to date");
            app_handle.emit(
                "create_modal",
                serde_json::json!({
                    "type": "notification",
                    "data": {
                        "title": "You're Up to Date",
                        "description": format!("You're running the latest version of Drop ({})", current_version),
                        "buttonText": "Great!"
                    }
                })
            ).unwrap();
        }
    } else {
        warn!("No tags found in GitHub repository");
    }

    Ok(())
} 
