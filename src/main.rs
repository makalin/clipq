use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod database;
mod daemon;
mod picker;
mod clipboard;
mod plugins;

use config::Config;
use database::Database;
use daemon::Daemon;

#[derive(Parser)]
#[command(name = "clipq")]
#[command(about = "Smart Clipboard Queue for power-users")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the clipboard daemon
    Daemon {
        /// Maximum number of clips to keep in history
        #[arg(short, long, default_value = "100")]
        max_clips: usize,
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Add text to clipboard and history
    Add {
        /// Text to add to clipboard
        text: String,
    },
    /// Pick and paste from history
    Pick {
        /// Maximum number of clips to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// List clipboard history
    List {
        /// Maximum number of clips to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Clear clipboard history
    Clear,
    /// Show configuration
    Config,
    /// Search clipboard history
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Show statistics
    Stats,
    /// Export clipboard history
    Export {
        /// Output file path
        #[arg(short, long, default_value = "clipboard_export.json")]
        output: String,
        /// Export format (json, csv, txt)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Import clipboard history
    Import {
        /// Input file path
        input: String,
        /// Import format (json, csv, txt)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Add file to clipboard
    File {
        /// File path to add
        path: String,
    },
    /// Show clipboard history with tags
    Tags {
        /// Tag to filter by
        tag: Option<String>,
    },
    /// Add tag to a clip
    Tag {
        /// Clip ID or index
        clip: String,
        /// Tag to add
        tag: String,
    },
    /// Remove tag from a clip
    Untag {
        /// Clip ID or index
        clip: String,
        /// Tag to remove
        tag: String,
    },
    /// Backup database
    Backup {
        /// Backup file path
        #[arg(short, long, default_value = "clipq_backup.db")]
        output: String,
    },
    /// Restore database
    Restore {
        /// Backup file path
        input: String,
    },
    /// Start web interface
    Web {
        /// Port to run web server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// List available plugins
    Plugins,
    /// Execute a plugin
    Plugin {
        /// Plugin name
        name: String,
        /// Input text
        input: String,
    },
    /// Extract URLs from text
    ExtractUrls {
        /// Text to extract URLs from
        text: String,
    },
    /// Generate password
    GeneratePassword {
        /// Password length
        #[arg(short, long, default_value = "16")]
        length: usize,
    },
    /// Calculate hash
    Hash {
        /// Text to hash
        text: String,
        /// Hash algorithm (md5, sha256, default)
        #[arg(short, long, default_value = "sha256")]
        algorithm: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { max_clips, config } => {
            let config_path = config.unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::env::current_dir().unwrap())
                    .join(".clipq.toml")
                    .to_string_lossy()
                    .to_string()
            });

            let config = Config::load(&config_path)?;
            let mut daemon = Daemon::new(config, max_clips).await?;
            daemon.run().await?;
        }
        Commands::Add { text } => {
            let mut db = Database::new().await?;
            let mut clipboard = clipboard::ClipboardManager::new()?;
            
            clipboard.set_text(&text)?;
            db.add_clip(&text, "text").await?;
            
            println!("Added to clipboard: {}", text);
        }
        Commands::Pick { limit } => {
            let mut db = Database::new().await?;
            let mut clipboard = clipboard::ClipboardManager::new()?;
            
            if let Some(selected) = picker::show_picker(&mut db, limit).await? {
                clipboard.set_text(&selected)?;
                println!("Pasted: {}", selected);
            }
        }
        Commands::List { limit } => {
            let db = Database::new().await?;
            let clips = db.get_recent_clips(limit).await?;
            
            for (i, clip) in clips.iter().enumerate() {
                println!("{}: {}", i + 1, clip.content);
            }
        }
        Commands::Clear => {
            let mut db = Database::new().await?;
            db.clear_history().await?;
            println!("Clipboard history cleared");
        }
        Commands::Config => {
            let config_path = dirs::home_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join(".clipq.toml");
            
            if config_path.exists() {
                let config = Config::load(&config_path.to_string_lossy())?;
                println!("Configuration loaded from: {}", config_path.display());
                println!("{:#?}", config);
            } else {
                println!("No configuration file found at: {}", config_path.display());
                println!("Creating default configuration...");
                let config = Config::default();
                config.save(&config_path.to_string_lossy())?;
                println!("Default configuration saved to: {}", config_path.display());
            }
        }
        Commands::Search { query, limit } => {
            let db = Database::new().await?;
            let clips = db.search_clips(&query, limit).await?;
            
            if clips.is_empty() {
                println!("No clips found matching '{}'", query);
            } else {
                println!("Found {} clips matching '{}':", clips.len(), query);
                for (i, clip) in clips.iter().enumerate() {
                    let preview = if clip.content.len() > 80 {
                        format!("{}...", &clip.content[..77])
                    } else {
                        clip.content.clone()
                    };
                    println!("{}: {}", i + 1, preview);
                }
            }
        }
        Commands::Stats => {
            let db = Database::new().await?;
            let stats = db.get_statistics().await?;
            
            println!("Clipboard Statistics");
            println!("===================");
            println!("Total clips: {}", stats.total_clips);
            println!("Text clips: {}", stats.text_clips);
            println!("File clips: {}", stats.file_clips);
            println!("Oldest clip: {}", stats.oldest_clip);
            println!("Newest clip: {}", stats.newest_clip);
            println!("Database size: {} KB", stats.db_size_kb);
        }
        Commands::Export { output, format } => {
            let db = Database::new().await?;
            let clips = db.get_all_clips().await?;
            
            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&clips)?;
                    std::fs::write(&output, json)?;
                    println!("Exported {} clips to {}", clips.len(), output);
                }
                "csv" => {
                    let mut csv = String::new();
                    csv.push_str("id,content,type,created_at,file_path\n");
                    let count = clips.len();
                    for clip in clips {
                        csv.push_str(&format!(
                            "{},{},{},{},{}\n",
                            clip.id,
                            clip.content.replace(',', "\\,"),
                            clip.clip_type,
                            clip.created_at.timestamp(),
                            clip.file_path.unwrap_or_default()
                        ));
                    }
                    std::fs::write(&output, csv)?;
                    println!("Exported {} clips to {}", count, output);
                }
                "txt" => {
                    let mut txt = String::new();
                    for (i, clip) in clips.iter().enumerate() {
                        txt.push_str(&format!("{}: {}\n", i + 1, clip.content));
                    }
                    std::fs::write(&output, txt)?;
                    println!("Exported {} clips to {}", clips.len(), output);
                }
                _ => {
                    println!("Unsupported format: {}. Use json, csv, or txt", format);
                }
            }
        }
        Commands::Import { input, format } => {
            let mut db = Database::new().await?;
            let content = std::fs::read_to_string(&input)?;
            
            match format.as_str() {
                "json" => {
                    let clips: Vec<crate::database::Clip> = serde_json::from_str(&content)?;
                    let count = clips.len();
                    for clip in clips {
                        db.add_clip(&clip.content, &clip.clip_type).await?;
                    }
                    println!("Imported {} clips from {}", count, input);
                }
                "csv" => {
                    let mut lines = content.lines();
                    lines.next(); // Skip header
                    let mut count = 0;
                    for line in lines {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 3 {
                            let content = parts[1].replace("\\,", ",");
                            let clip_type = parts[2];
                            db.add_clip(&content, clip_type).await?;
                            count += 1;
                        }
                    }
                    println!("Imported {} clips from {}", count, input);
                }
                "txt" => {
                    let mut count = 0;
                    for line in content.lines() {
                        if !line.trim().is_empty() {
                            db.add_clip(line.trim(), "text").await?;
                            count += 1;
                        }
                    }
                    println!("Imported {} clips from {}", count, input);
                }
                _ => {
                    println!("Unsupported format: {}. Use json, csv, or txt", format);
                }
            }
        }
        Commands::File { path } => {
            let mut db = Database::new().await?;
            let mut clipboard = clipboard::ClipboardManager::new()?;
            
            if std::path::Path::new(&path).exists() {
                let abs_path = std::fs::canonicalize(&path)?;
                let path_str = abs_path.to_string_lossy();
                
                clipboard.set_text(&path_str)?;
                db.add_file_clip(&path_str).await?;
                
                println!("Added file to clipboard: {}", path_str);
            } else {
                println!("File not found: {}", path);
            }
        }
        Commands::Tags { tag } => {
            let db = Database::new().await?;
            let clips = if let Some(tag) = tag {
                db.get_clips_by_tag(&tag).await?
            } else {
                db.get_all_clips().await?
            };
            
            for (i, clip) in clips.iter().enumerate() {
                let tags = db.get_clip_tags(&clip.id).await?;
                let tag_str = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", tags.join(", "))
                };
                println!("{}: {}{}", i + 1, clip.content, tag_str);
            }
        }
        Commands::Tag { clip, tag } => {
            let mut db = Database::new().await?;
            
            // Try to parse as index first, then as ID
            let clip_id = if let Ok(index) = clip.parse::<usize>() {
                let clips = db.get_recent_clips(index).await?;
                if index > 0 && index <= clips.len() {
                    clips[index - 1].id.clone()
                } else {
                    println!("Invalid clip index: {}", index);
                    return Ok(());
                }
            } else {
                clip.clone()
            };
            
            db.add_tag_to_clip(&clip_id, &tag).await?;
            println!("Added tag '{}' to clip {}", tag, clip_id);
        }
        Commands::Untag { clip, tag } => {
            let mut db = Database::new().await?;
            
            // Try to parse as index first, then as ID
            let clip_id = if let Ok(index) = clip.parse::<usize>() {
                let clips = db.get_recent_clips(index).await?;
                if index > 0 && index <= clips.len() {
                    clips[index - 1].id.clone()
                } else {
                    println!("Invalid clip index: {}", index);
                    return Ok(());
                }
            } else {
                clip.clone()
            };
            
            db.remove_tag_from_clip(&clip_id, &tag).await?;
            println!("Removed tag '{}' from clip {}", tag, clip_id);
        }
        Commands::Backup { output } => {
            let db = Database::new().await?;
            db.backup(&output).await?;
            println!("Database backed up to: {}", output);
        }
        Commands::Restore { input } => {
            let mut db = Database::new().await?;
            db.restore(&input).await?;
            println!("Database restored from: {}", input);
        }
        Commands::Web { port } => {
            println!("Web interface temporarily disabled.");
            println!("Would start web server on port: {}", port);
            println!("This feature will be re-enabled in a future update.");
        }
        Commands::Plugins => {
            let db = Arc::new(Mutex::new(Database::new().await?));
            let mut plugin_manager = plugins::PluginManager::new(db);
            plugin_manager.load_plugins()?;
            
            println!("Available Plugins:");
            println!("==================");
            for plugin in plugin_manager.list_plugins() {
                let status = if plugin.enabled { "enabled" } else { "disabled" };
                println!("{} - {} ({})", plugin.name, plugin.command, status);
            }
        }
        Commands::Plugin { name, input } => {
            let db = Arc::new(Mutex::new(Database::new().await?));
            let mut plugin_manager = plugins::PluginManager::new(db);
            plugin_manager.load_plugins()?;
            
            let result = plugin_manager.execute_plugin(&name, &input).await?;
            print!("{}", result);
        }
        Commands::ExtractUrls { text } => {
            let urls = plugins::builtin::extract_urls(&text);
            if urls.is_empty() {
                println!("No URLs found in text");
            } else {
                println!("Found {} URLs:", urls.len());
                for url in urls {
                    println!("  {}", url);
                }
            }
        }
        Commands::GeneratePassword { length } => {
            let password = plugins::builtin::generate_password(length);
            println!("Generated password: {}", password);
        }
        Commands::Hash { text, algorithm } => {
            let hash = plugins::builtin::calculate_hash(&text, &algorithm);
            println!("{} hash: {}", algorithm, hash);
        }
    }

    Ok(())
}