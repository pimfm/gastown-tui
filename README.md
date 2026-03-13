# Gas Town TUI

A terminal user interface for [Gas Town](https://github.com/steveyegge/gastown) — Steve Yegge's multi-agent orchestration system for Claude Code.

Built with Rust and [Ratatui](https://ratatui.rs).

## Features

- **Dashboard** — stat cards, convoy progress bars, agent status grid, live activity feed
- **Agents** — table view of all 40+ agents with status, runtime, rig, current bead, and uptime
- **Convoys** — progress tracking for all work convoys with assigned agent lists
- **Beads** — full issue tracker view with status, assignment, and convoy linkage
- **Repos** — scans your system for git repositories with branch, dirty status, and sync info
- **Spawn dialog** — quickly spin up new agents with rig/runtime/task selection
- **Search/filter** — press `/` to filter any tab by keyword
- **Live refresh** — auto-refreshes every 5 seconds, manual refresh with `r`

## Requirements

- Rust 1.75+
- Gas Town (`gt`) and Beads (`bd`) CLIs are optional — runs in demo mode without them

## Install

```sh
cargo install --path .
```

## Usage

```sh
gastown-tui
```

## Keybindings

| Key | Action |
|-----|--------|
| `1-5` | Switch to tab |
| `Tab` / `Shift+Tab` | Next/previous tab |
| `j` / `k` / `↑` / `↓` | Scroll up/down |
| `g` / `G` | Jump to top/bottom |
| `/` | Toggle search filter |
| `s` | Spawn new agent |
| `r` | Refresh data |
| `q` / `Ctrl+C` | Quit |

## Architecture

When Gas Town CLI tools (`gt`, `bd`) are available, the TUI shells out to them for live data. Without them it runs in demo mode with realistic sample data showing 40 agents across 8 rigs.

The repo scanner searches `~/vibe`, `~/code`, `~/fm`, `~/solo`, and `~/flock` for git repositories.
