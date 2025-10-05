use anyhow::Result;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::database::Database;
// use crate::picker; // TODO: Re-enable when hotkey support is added back

pub struct Daemon {
    config: Config,
    max_clips: usize,
    db: Arc<Mutex<Database>>,
    clipboard: Arc<Mutex<ClipboardManager>>,
    hotkey_manager: Option<GlobalHotKeyManager>,
}

impl Daemon {
    pub async fn new(config: Config, max_clips: usize) -> Result<Self> {
        let db = Arc::new(Mutex::new(Database::new().await?));
        let clipboard = Arc::new(Mutex::new(ClipboardManager::new()?));
        
        let mut daemon = Self {
            config,
            max_clips,
            db,
            clipboard,
            hotkey_manager: None,
        };
        
        daemon.setup_hotkey().await?;
        Ok(daemon)
    }

    async fn setup_hotkey(&mut self) -> Result<()> {
        // For now, skip hotkey setup to focus on core functionality
        // TODO: Implement proper hotkey handling
        info!("Hotkey support temporarily disabled - use 'clipq pick' command instead");
        Ok(())
    }

    fn parse_hotkey(&self, hotkey_str: &str) -> Result<HotKey> {
        let parts: Vec<&str> = hotkey_str.split('+').collect();
        let mut modifiers = Modifiers::empty();
        let mut key_code = Code::KeyV; // Default to V key

        for part in parts {
            match part.trim().to_lowercase().as_str() {
                "ctrl" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                "meta" | "cmd" | "super" => modifiers |= Modifiers::META,
                "v" => key_code = Code::KeyV,
                "c" => key_code = Code::KeyC,
                "x" => key_code = Code::KeyX,
                _ => {
                    // Try to parse as a single character
                    if let Some(ch) = part.chars().next() {
                        if ch.is_ascii_alphabetic() {
                            key_code = match ch.to_ascii_uppercase() {
                                'A' => Code::KeyA,
                                'B' => Code::KeyB,
                                'C' => Code::KeyC,
                                'D' => Code::KeyD,
                                'E' => Code::KeyE,
                                'F' => Code::KeyF,
                                'G' => Code::KeyG,
                                'H' => Code::KeyH,
                                'I' => Code::KeyI,
                                'J' => Code::KeyJ,
                                'K' => Code::KeyK,
                                'L' => Code::KeyL,
                                'M' => Code::KeyM,
                                'N' => Code::KeyN,
                                'O' => Code::KeyO,
                                'P' => Code::KeyP,
                                'Q' => Code::KeyQ,
                                'R' => Code::KeyR,
                                'S' => Code::KeyS,
                                'T' => Code::KeyT,
                                'U' => Code::KeyU,
                                'V' => Code::KeyV,
                                'W' => Code::KeyW,
                                'X' => Code::KeyX,
                                'Y' => Code::KeyY,
                                'Z' => Code::KeyZ,
                                _ => Code::KeyV,
                            };
                        }
                    }
                }
            }
        }

        Ok(HotKey::new(Some(modifiers), key_code))
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting clipq daemon with max_clips={}", self.max_clips);
        
        // Start clipboard monitoring
        let db_clone = Arc::clone(&self.db);
        let max_clips = self.max_clips;
        let clipboard_clone = Arc::clone(&self.clipboard);
        
        let monitor_task = tokio::spawn(async move {
            let mut clipboard = clipboard_clone.lock().await;
            let mut last_content = None;
            
            loop {
                if let Ok(Some(content)) = clipboard.get_text() {
                    if last_content.as_ref() != Some(&content) && !content.trim().is_empty() {
                        last_content = Some(content.clone());
                        
                        let mut db = db_clone.lock().await;
                        if let Err(e) = db.add_clip(&content, "text").await {
                            error!("Failed to add clip to database: {}", e);
                        } else {
                            // Trim history to max_clips
                            if let Err(e) = db.trim_history(max_clips).await {
                                error!("Failed to trim history: {}", e);
                            }
                        }
                    }
                }
                
                sleep(Duration::from_millis(500)).await;
            }
        });

        // For now, just run clipboard monitoring
        // TODO: Add hotkey support back
        monitor_task.await?;

        Ok(())
    }
}