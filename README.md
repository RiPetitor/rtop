# rtop

Terminal-based system monitor for Linux with GPU support.

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Linux-green)

## Features

- **Process monitoring** — CPU, memory, uptime, status
- **GPU support** — NVIDIA (nvidia-smi), AMD, Intel (sysfs/lspci)
- **VRAM tracking** — Real-time GPU memory usage
- **Interactive** — Sort, navigate, terminate processes
- **Configurable** — CLI args and config file
- **Lightweight** — Minimal dependencies, fast startup

## Installation

```bash
# Clone and build
git clone https://github.com/yourusername/rtop.git
cd rtop
cargo build --release

# Install
cargo install --path .
```

### Optional: PCI device names

For human-readable GPU names instead of hex IDs:

```bash
cargo build --release --features pci-names
```

Requires `libpci-dev` / `pciutils-devel` package.

## Usage

```bash
rtop [options]
```

### Options

| Option | Description |
|--------|-------------|
| `--tick-ms <ms>` | Refresh interval (default: 1000, min: 100) |
| `--no-vram` | Disable GPU probing |
| `--sort <key>` | Sort by: `pid`, `cpu`, `mem`, `uptime`, `stat`, `name` |
| `--sort-dir <dir>` | Sort direction: `asc`, `desc` |
| `--gpu <pref>` | GPU preference: `auto`, `discrete`, `integrated` |
| `-h, --help` | Show help |

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `↑` / `↓` | Navigate processes |
| `←` / `→` | Change sort column |
| `Space` | Toggle sort direction |
| `Enter` | Terminate process (with confirmation) |
| `c` / `m` / `p` / `n` | Quick sort by CPU/Mem/PID/Name |
| `g` / `G` | Next/Previous GPU |
| `r` | Force refresh |

## Configuration

Config file: `~/.config/rtop/config.toml`

```toml
[general]
tick_rate_ms = 1000
gpu_poll_ms = 2000

[display]
show_vram = true
default_sort = "cpu"
sort_dir = "desc"
gpu_preference = "auto"
```

CLI arguments override config file settings.

## Architecture

```
src/
├── main.rs              # Entry point
├── lib.rs               # Public API
├── error.rs             # Error types
├── app/                 # Application state & config
├── data/                # Data models
│   └── gpu/             # GPU providers (nvidia, lspci, sysfs)
├── events/              # Event handling
├── ui/                  # TUI rendering
└── utils/               # Helpers
```

### GPU Detection

rtop uses multiple sources for GPU detection:

1. **nvidia-smi** — NVIDIA GPUs with VRAM info
2. **lspci** — PCI device enumeration
3. **sysfs** — `/sys/class/drm` for AMD VRAM

Results are merged with priority to nvidia-smi for NVIDIA cards.

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal handling
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — System information
- [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml) — Config parsing
- [thiserror](https://github.com/dtolnay/thiserror) — Error handling

## License

MIT
