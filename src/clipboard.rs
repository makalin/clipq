use anyhow::Result;
use arboard::Clipboard as ArboardClipboard;
use std::time::Duration;
use tokio::time::sleep;

pub struct ClipboardManager {
    clipboard: ArboardClipboard,
    last_content: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = ArboardClipboard::new()?;
        Ok(Self {
            clipboard,
            last_content: None,
        })
    }

    pub fn get_text(&mut self) -> Result<Option<String>> {
        match self.clipboard.get_text() {
            Ok(text) => Ok(Some(text)),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)?;
        self.last_content = Some(text.to_string());
        Ok(())
    }

    pub fn get_image(&mut self) -> Result<Option<arboard::ImageData>> {
        match self.clipboard.get_image() {
            Ok(image) => Ok(Some(image)),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_image(&mut self, image: arboard::ImageData) -> Result<()> {
        self.clipboard.set_image(image)?;
        Ok(())
    }

    pub async fn monitor_changes<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(String) -> Result<()>,
    {
        loop {
            if let Ok(Some(content)) = self.get_text() {
                // Only process if content has changed
                if self.last_content.as_ref() != Some(&content) {
                    self.last_content = Some(content.clone());
                    if let Err(e) = callback(content) {
                        log::error!("Error processing clipboard content: {}", e);
                    }
                }
            }
            
            // Check for clipboard changes every 500ms
            sleep(Duration::from_millis(500)).await;
        }
    }

    pub fn has_changed(&mut self) -> Result<bool> {
        if let Ok(Some(content)) = self.get_text() {
            let changed = self.last_content.as_ref() != Some(&content);
            if changed {
                self.last_content = Some(content);
            }
            Ok(changed)
        } else {
            Ok(false)
        }
    }
}

// Re-export for convenience
pub type Clipboard = ClipboardManager;