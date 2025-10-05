use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{Database, Clip};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
    pub trigger: PluginTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginTrigger {
    OnClipAdd,
    OnClipSearch,
    OnClipPick,
    OnDaemonStart,
    OnDaemonStop,
    Manual,
}

pub struct PluginManager {
    plugins: HashMap<String, PluginConfig>,
    db: Arc<Mutex<Database>>,
}

impl PluginManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            plugins: HashMap::new(),
            db,
        }
    }

    pub fn load_plugins(&mut self) -> Result<()> {
        // Load built-in plugins
        self.add_plugin(PluginConfig {
            name: "url_extractor".to_string(),
            command: "python3".to_string(),
            args: vec!["-c".to_string(), "import re, sys; print('\\n'.join(re.findall(r'https?://[^\\s]+', sys.stdin.read())))".to_string()],
            enabled: true,
            trigger: PluginTrigger::OnClipAdd,
        })?;

        self.add_plugin(PluginConfig {
            name: "code_formatter".to_string(),
            command: "python3".to_string(),
            args: vec!["-c".to_string(), "import json, sys; data=json.loads(sys.stdin.read()); print(json.dumps(data, indent=2))".to_string()],
            enabled: true,
            trigger: PluginTrigger::OnClipAdd,
        })?;

        self.add_plugin(PluginConfig {
            name: "password_generator".to_string(),
            command: "python3".to_string(),
            args: vec!["-c".to_string(), "import secrets, string; print(''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(16)))".to_string()],
            enabled: true,
            trigger: PluginTrigger::Manual,
        })?;

        Ok(())
    }

    pub fn add_plugin(&mut self, plugin: PluginConfig) -> Result<()> {
        self.plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    pub async fn execute_plugin(&self, plugin_name: &str, input: &str) -> Result<String> {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_name))?;

        if !plugin.enabled {
            return Err(anyhow::anyhow!("Plugin is disabled: {}", plugin_name));
        }

        let output = Command::new(&plugin.command)
            .args(&plugin.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let mut child = output;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Plugin execution failed: {}", error));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn trigger_plugins(&self, trigger: &PluginTrigger, clip: &Clip) -> Result<()> {
        for (name, plugin) in &self.plugins {
            if !plugin.enabled {
                continue;
            }

            match (&plugin.trigger, trigger) {
                (PluginTrigger::OnClipAdd, PluginTrigger::OnClipAdd) => {
                    if let Err(e) = self.execute_plugin(name, &clip.content).await {
                        log::warn!("Plugin {} failed: {}", name, e);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<&PluginConfig> {
        self.plugins.values().collect()
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", name))
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", name))
        }
    }
}

// Built-in plugins
pub mod builtin {
    use super::*;

    pub fn extract_urls(text: &str) -> Vec<String> {
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        url_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn extract_emails(text: &str) -> Vec<String> {
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        email_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn extract_phone_numbers(text: &str) -> Vec<String> {
        let phone_regex = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap();
        phone_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn format_json(text: &str) -> Result<String> {
        let parsed: serde_json::Value = serde_json::from_str(text)?;
        Ok(serde_json::to_string_pretty(&parsed)?)
    }

    pub fn generate_password(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    pub fn calculate_hash(text: &str, algorithm: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        match algorithm {
            "sha256" => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(text.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            _ => {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            }
        }
    }
}