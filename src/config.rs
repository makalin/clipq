use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub max_clips: usize,
    pub hotkey: String,
    pub picker_command: String,
    pub database_path: String,
    pub enable_file_clips: bool,
    pub enable_encryption: bool,
    pub sync_enabled: bool,
    pub sync_gist_id: Option<String>,
    pub sync_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_clips: 100,
            hotkey: "ctrl+shift+v".to_string(),
            picker_command: "fzf".to_string(),
            database_path: "~/.clipq/clipboard.db".to_string(),
            enable_file_clips: true,
            enable_encryption: false,
            sync_enabled: false,
            sync_gist_id: None,
            sync_token: None,
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let expanded_path = shellexpand::tilde(path).to_string();
        
        if Path::new(&expanded_path).exists() {
            let content = fs::read_to_string(&expanded_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let expanded_path = shellexpand::tilde(path).to_string();
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&expanded_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&expanded_path, content)?;
        Ok(())
    }
}