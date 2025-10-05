use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command as AsyncCommand;
use which::which;

use crate::database::Database;

pub async fn show_picker(db: &mut Database, limit: usize) -> Result<Option<String>> {
    let clips = db.get_recent_clips(limit).await?;
    
    if clips.is_empty() {
        println!("No clipboard history found");
        return Ok(None);
    }

    // Try to find fzf or skim
    let picker_cmd = find_picker_command()?;
    
    // Prepare input for the picker
    let input = clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            let preview = if clip.content.len() > 100 {
                format!("{}...", &clip.content[..97])
            } else {
                clip.content.clone()
            };
            format!("{}: {}", i + 1, preview)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let result = run_picker(&picker_cmd, &input).await?;
    
    if let Some(selected_line) = result {
        // Extract the content from the selected line
        if let Some(colon_pos) = selected_line.find(':') {
            let index_str = &selected_line[..colon_pos];
            if let Ok(index) = index_str.parse::<usize>() {
                if index > 0 && index <= clips.len() {
                    return Ok(Some(clips[index - 1].content.clone()));
                }
            }
        }
    }

    Ok(None)
}

fn find_picker_command() -> Result<String> {
    // Try fzf first
    if which("fzf").is_ok() {
        return Ok("fzf".to_string());
    }
    
    // Try skim
    if which("sk").is_ok() {
        return Ok("sk".to_string());
    }
    
    // Try skim with full name
    if which("skim").is_ok() {
        return Ok("skim".to_string());
    }
    
    // Fallback to a simple menu
    Err(anyhow::anyhow!(
        "No fuzzy picker found. Please install 'fzf' or 'skim' (sk).\n\
        Install fzf: https://github.com/junegunn/fzf\n\
        Install skim: https://github.com/lotabout/skim"
    ))
}

async fn run_picker(cmd: &str, input: &str) -> Result<Option<String>> {
    let mut command = match cmd {
        "fzf" => {
            let mut cmd = AsyncCommand::new("fzf");
            cmd.args(&["--height", "40%", "--reverse", "--border"]);
            cmd
        }
        "sk" | "skim" => {
            let mut cmd = AsyncCommand::new(cmd);
            cmd.args(&["--height", "40%", "--reverse", "--border"]);
            cmd
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown picker command: {}", cmd));
        }
    };

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;
    
    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(input.as_bytes()).await?;
    }

    let output = child.wait_with_output().await?;
    
    if output.status.success() {
        let result = String::from_utf8(output.stdout)?;
        if result.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(result.trim().to_string()))
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("User aborted") || error.contains("cancelled") {
            Ok(None)
        } else {
            Err(anyhow::anyhow!("Picker command failed: {}", error))
        }
    }
}

pub async fn show_simple_menu(db: &mut Database, limit: usize) -> Result<Option<String>> {
    let clips = db.get_recent_clips(limit).await?;
    
    if clips.is_empty() {
        println!("No clipboard history found");
        return Ok(None);
    }

    println!("\nClipboard History:");
    println!("==================");
    
    for (i, clip) in clips.iter().enumerate() {
        let preview = if clip.content.len() > 80 {
            format!("{}...", &clip.content[..77])
        } else {
            clip.content.clone()
        };
        println!("{}: {}", i + 1, preview);
    }
    
    println!("\nEnter number to select (0 to cancel): ");
    
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice: usize = input.trim().parse().unwrap_or(0);
    
    if choice == 0 || choice > clips.len() {
        Ok(None)
    } else {
        Ok(Some(clips[choice - 1].content.clone()))
    }
}