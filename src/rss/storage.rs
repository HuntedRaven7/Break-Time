use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};

/*
 * RSS Feed Storage Manager (Senior Dev Persistence Edition).
 * Handles saving and loading the feed list to ~/.config/break-time/feeds.json.
 */

#[derive(Serialize, Deserialize, Debug)]
struct FeedConfig {
    pub feeds: Vec<String>,
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("io.github", "HuntedRaven7", "BreakTime") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        return Some(config_dir.join("feeds.json"));
    }
    None
}

pub fn load_feeds() -> Vec<String> {
    if let Some(path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<FeedConfig>(&content) {
                return config.feeds;
            }
        }
    }
    // Default feeds if file doesn't exist or fails to load
    vec!["https://www.reddit.com/.rss".to_string()]
}

pub fn save_feeds(feeds: Vec<String>) {
    if let Some(path) = get_config_path() {
        let config = FeedConfig { feeds };
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = fs::write(path, json) {
                eprintln!("Failed to save feeds to disk: {}", e);
            }
        }
    }
}
