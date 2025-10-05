# clipq

Smart Clipboard Queue for power-users.  
Never lose your copy history again.

## Problem
Native clipboards store only one item. Reboot, crash, or just copy twice—and your history is gone.  
For developers, writers, and multitaskers, this kills flow.

## Solution
`clipq` is a tiny cross-platform clipboard daemon written in **Rust**.  
It keeps the last **N clips** (text & files) in a local **SQLite DB**.  
Hit **Ctrl+Shift+V** to open a fuzzy picker (fzf/skim) and paste instantly.

## Features
- Persistent clipboard history (survives reboot)
- Supports text & file paths
- Configurable history size (N)
- Lightweight SQLite backend
- Global hotkey (**Ctrl+Shift+V**)
- Fuzzy search picker (fzf/skim)
- Optional end-to-end encrypted sync via GitHub Gist

## Stack
- Rust (cross-platform clipboard crate)
- SQLite (lightweight, embedded)
- fzf / skim for fuzzy search UI

## Installation

### Quick Install
```bash
# Clone the repository
git clone https://github.com/yourusername/clipq.git
cd clipq

# Build and install
./install.sh
```

### Manual Build
```bash
# Build the project
cargo build --release

# Or use the build script
./build.sh
```

## Usage

### Basic Commands
```bash
# Run the daemon (monitors clipboard automatically)
clipq daemon

# Add text to clipboard and history
clipq add "Hello, World!"

# Pick and paste from history (requires fzf or skim)
clipq pick

# List clipboard history
clipq list

# Clear clipboard history
clipq clear

# Show configuration
clipq config
```

### Daemon Mode
```bash
# Run the daemon with custom settings
clipq daemon --max-clips 200 --config ~/.clipq.toml
```

### Configuration
The configuration file is automatically created at `~/.clipq.toml`:

```toml
max_clips = 100
hotkey = "ctrl+shift+v"
picker_command = "fzf"
database_path = "~/.clipq/clipboard.db"
enable_file_clips = true
enable_encryption = false
sync_enabled = false
```

## Testing
```bash
# Run the test suite
./test.sh
```

## Roadmap

* [x] Config file (`~/.clipq.toml`)
* [x] SQLite database backend
* [x] Cross-platform clipboard support
* [x] Fuzzy picker integration (fzf/skim)
* [x] CLI interface
* [x] Daemon mode
* [ ] Global hotkey support (temporarily disabled)
* [ ] Multi-device sync (encrypted Gist)
* [ ] File preview in picker
* [ ] GUI tray app (optional)

## Why clipq?

* **Tiny:** single binary, no deps
* **Safe:** local DB by default, optional encrypted sync
* **Fast:** Rust + fzf, made for speed

---

💡 Inspired by productivity power-users. Built to never lose a clip again.
