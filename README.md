# nm — Network Monitor

Lightweight terminal-based network traffic monitor for macOS. Displays real-time per-process bandwidth usage, individual connections with reverse DNS, and bandwidth sparklines.

![Rust](https://img.shields.io/badge/Rust-2021-orange) ![macOS](https://img.shields.io/badge/platform-macOS-blue)

<img width="753" height="823" alt="image" src="https://github.com/user-attachments/assets/ec1418ea-93ba-4c92-b9b1-db05c404ea50" />



## Features

- **Processes tab** — per-process download/upload totals and rates with visual rate bars
- **Connections tab** — all active TCP/UDP connections with local/remote addresses and reverse DNS hostnames
- **Overview tab** — aggregate stats, top 10 processes by rate, bandwidth sparkline history
- **Filtering** — case-insensitive search across process names, paths, PIDs, and connection addresses
- **Drill-down** — press Enter on a process to jump to its connections
- **Sorting** — cycle through 7 sort fields (name, PID, connections, down, up, rate-in, rate-out)
- **Pause/resume** — freeze data collection while reviewing

## Requirements

- macOS (uses `nettop` and `libproc`)
- Rust 1.70+ (edition 2021)

## Installation

```bash
git clone https://github.com/0x2e8b/nm.git
cd nm
cargo build --release
```

The binary will be at `target/release/nm`.

## Usage

```bash
# Basic usage (requires root for full nettop access)
sudo ./target/release/nm

# Custom refresh interval (5 seconds) and initial sort
sudo ./target/release/nm -i 5 -s conn
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --interval` | Refresh interval in seconds | 2 |
| `-s, --sort-by` | Initial sort: name, pid, conn, down, up, rate-in, rate-out | rate-in |

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift-Tab` | Switch tabs |
| `j` / `k` / `↑` / `↓` | Navigate rows |
| `Enter` | Drill into process connections |
| `s` | Cycle sort field |
| `/` | Filter (type query, Enter to apply) |
| `Esc` | Clear filter / close help |
| `p` | Pause/resume data collection |
| `?` | Help overlay |
| `q` | Quit |

## How It Works

`nm` periodically runs macOS `nettop` to capture per-process network statistics with per-connection detail. It computes bandwidth rates by diffing consecutive snapshots, enriches connections with reverse DNS lookups (async, non-blocking), and resolves executable paths via `libproc`. All data is displayed in a ratatui-powered TUI with three tabs.

## License

MIT
