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

## Usage
```bash
# Run the daemon
clipq --daemon

# Add to clipboard (auto-captured)
echo "hello" | clipq

# Pick and paste from history
clipq --pick
````

## Roadmap

* [ ] Config file (`~/.clipq.toml`)
* [ ] Multi-device sync (encrypted Gist)
* [ ] File preview in picker
* [ ] GUI tray app (optional)

## Why clipq?

* **Tiny:** single binary, no deps
* **Safe:** local DB by default, optional encrypted sync
* **Fast:** Rust + fzf, made for speed

---

💡 Inspired by productivity power-users. Built to never lose a clip again.
