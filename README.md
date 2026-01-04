# rtop

[EN](#english) | [RU](#ru)

![Rust](https://img.shields.io/badge/Rust-1.85+-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Linux-green)

<a id="english"></a>
## English

Terminal system monitor for Linux with GPU and container support.

### Features

- **Processes** — CPU, memory, uptime, status + process tree
- **Sorting and filters** — hotkeys + mouse column sorting
- **GPU** — NVIDIA (nvidia-smi), AMD/Intel (sysfs/lspci)
- **GPU processes** — per-process load/VRAM (nvidia-smi, DRM fdinfo)
- **VRAM** — realtime GPU memory usage
- **System tab** — extended info
- **Containers** — list, net rate and drill-down into processes
- **Setup/Help** — modal windows (F2/F12) + language toggle (EN/RU)

### Installation

Requires Rust 1.85+.

```bash
# Clone and build
git clone https://github.com/RiPetitor/rtop.git
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

Requires `libpci-dev` / `pciutils-devel`.

### Usage

```bash
rtop [options]
```

Minimum terminal size: 60x22.

### Options

| Option | Description |
|------|----------|
| `--tick-ms <ms>` | Refresh interval (default 1000, min 100) |
| `--no-vram` | Disable GPU probing |
| `--sort <key>` | Sorting: `pid`, `user`, `cpu`, `mem`, `uptime`, `stat`, `name` |
| `--sort-dir <dir>` | Direction: `asc`, `desc` |
| `--gpu <pref>` | GPU preference: `auto`, `discrete`, `integrated` |
| `-h, --help` | Show help |

### Hotkeys

| Key | Action |
|--------|----------|
| `q` / `Ctrl+C` | Quit |
| `↑` / `↓` | Navigate processes |
| `←` / `→` | Change sort column |
| `Space` | Toggle sort direction |
| `Enter` | Action (terminate process / open container) |
| `c` / `m` / `p` / `n` / `u` | Quick sort CPU/Mem/PID/Name/User |
| `h` | Highlight processes (user/non-root/GUI) |
| `g` / `G` | Next/previous GPU |
| `t` | Process tree (Processes/Overview only) |
| `1` / `2` / `3` / `4` / `5` | Overview / System / GPU / Containers / Processes |
| `Tab` | Cycle views (Overview → Processes → GPU → System → Containers) |
| `b` / `Esc` | Back from container drill-down |
| `F2` | Setup |
| `F12` | Help |
| `r` | Force refresh |

### Mouse

- Left click column header — sort by column / toggle direction.
- In tree mode, sorting is fixed by PID.

### Configuration

File: `~/.config/rtop/config.toml`

```toml
[general]
tick_rate_ms = 1000
gpu_poll_ms = 2000

[display]
show_vram = true
default_sort = "cpu"
sort_dir = "desc"
gpu_preference = "auto"
language = "en"
logo_mode = "ascii"
logo_quality = "medium"
```

CLI args override the config.
Display settings are saved to the config when toggled in Setup (language, logo mode, logo quality).

Display options:
- `logo_mode`: `ascii` or `svg`
- `logo_quality`: `quality` (Smoothed), `medium` (Medium), `pixel` (Detailed)

### Custom logo

1. Create folders:
   - `~/.config/rtop/logo/ascii/`
   - `~/.config/rtop/logo/svg/`
2. Put your logo file in one of the folders.
   - The first file in alphabetical order is used.
   - ASCII: any text file, colors via `$1..$9`, reset with `$0`, literal `$` with `$$`.
3. Optional palette file in `~/.config/rtop/logo/`:
   - `palette.json`, `palette.yaml`, or `palette.yml`
   - RGB values are 0-255.

Example `palette.json`:

```json
{
  "default": [255, 255, 255],
  "colors": [
    [78, 190, 210],
    [230, 180, 70],
    [230, 90, 70]
  ]
}
```

### Architecture

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

### GPU detection

rtop uses multiple sources:

1. **nvidia-smi** — NVIDIA GPUs with VRAM
2. **lspci** — PCI enumeration
3. **sysfs** — `/sys/class/drm` for AMD/Intel

Results are merged with nvidia-smi priority for NVIDIA.
GPU processes use `nvidia-smi pmon` and `/proc/*/fdinfo` (DRM).

### Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal handling
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — System information
- [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml) — Config parsing
- [thiserror](https://github.com/dtolnay/thiserror) — Error handling

### License

MIT

<a id="ru"></a>
## Русский

Терминальный монитор системы для Linux с поддержкой GPU и контейнеров.

### Возможности

- **Процессы** — CPU, память, аптайм, статус + дерево процессов
- **Сортировка и фильтры** — быстрые клавиши + клики мышью по заголовкам
- **GPU** — NVIDIA (nvidia-smi), AMD/Intel (sysfs/lspci)
- **GPU процессы** — загрузка/VRAM по процессам (nvidia-smi, DRM fdinfo)
- **VRAM** — использование памяти видеокарты в реальном времени
- **Системная вкладка** — расширенная информация
- **Контейнеры** — список контейнеров, net‑rate и drill‑down в процессы
- **Setup/Help** — модальные окна (F2/F12) + переключение языка (EN/RU)

### Установка

Требуется Rust 1.85+.

```bash
# Clone and build
git clone https://github.com/RiPetitor/rtop.git
cd rtop
cargo build --release

# Install
cargo install --path .
```

### Опционально: PCI названия устройств

Для человекочитаемых имён GPU вместо hex ID:

```bash
cargo build --release --features pci-names
```

Нужен пакет `libpci-dev` / `pciutils-devel`.

### Использование

```bash
rtop [options]
```

Минимальный размер терминала: 60x22.

### Опции

| Опция | Описание |
|------|----------|
| `--tick-ms <ms>` | Интервал обновления (по умолчанию 1000, минимум 100) |
| `--no-vram` | Отключить GPU probing |
| `--sort <key>` | Сортировка: `pid`, `user`, `cpu`, `mem`, `uptime`, `stat`, `name` |
| `--sort-dir <dir>` | Направление: `asc`, `desc` |
| `--gpu <pref>` | GPU предпочтение: `auto`, `discrete`, `integrated` |
| `-h, --help` | Показать справку |

### Горячие клавиши

| Клавиша | Действие |
|--------|----------|
| `q` / `Ctrl+C` | Выход |
| `↑` / `↓` | Навигация по процессам |
| `←` / `→` | Смена колонки сортировки |
| `Space` | Переключить направление сортировки |
| `Enter` | Действие (завершить процесс / открыть контейнер) |
| `c` / `m` / `p` / `n` / `u` | Быстрая сортировка CPU/Mem/PID/Name/User |
| `h` | Подсветка процессов (user/non‑root/GUI) |
| `g` / `G` | Следующий/предыдущий GPU |
| `t` | Дерево процессов (только в Processes/Overview) |
| `1` / `2` / `3` / `4` / `5` | Обзор / Система / GPU / Контейнеры / Процессы |
| `Tab` | Циклическое переключение вкладок (Обзор → Процессы → GPU → Система → Контейнеры) |
| `b` / `Esc` | Назад из контейнерного drill‑down |
| `F2` | Setup |
| `F12` | Help |
| `r` | Принудительное обновление |

### Мышь

- ЛКМ по заголовку колонки — сортировка по колонке / смена направления.
- В режиме дерева сортировка фиксирована по PID.

### Конфигурация

Файл: `~/.config/rtop/config.toml`

```toml
[general]
tick_rate_ms = 1000
gpu_poll_ms = 2000

[display]
show_vram = true
default_sort = "cpu"
sort_dir = "desc"
gpu_preference = "auto"
language = "en"
logo_mode = "ascii"
logo_quality = "medium"
```

CLI‑аргументы имеют приоритет над конфигом.
Параметры отображения сохраняются в конфиге при переключении в Setup (язык, режим лого, качество лого).

Опции отображения:
- `logo_mode`: `ascii` или `svg`
- `logo_quality`: `quality` (Сглаженный), `medium` (Средне), `pixel` (Детальный)

### Свой логотип

1. Создайте папки:
   - `~/.config/rtop/logo/ascii/`
   - `~/.config/rtop/logo/svg/`
2. Положите файл логотипа в нужную папку.
   - Берётся первый файл по алфавиту.
   - ASCII: любой текстовый файл, цвета через `$1..$9`, сброс `$0`, литерал `$` - `$$`.
3. (Опционально) палитра в `~/.config/rtop/logo/`:
   - `palette.json`, `palette.yaml`, или `palette.yml`
   - RGB значения 0-255.

Пример `palette.json`:

```json
{
  "default": [255, 255, 255],
  "colors": [
    [78, 190, 210],
    [230, 180, 70],
    [230, 90, 70]
  ]
}
```

### Архитектура

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

### GPU детекция

rtop использует несколько источников:

1. **nvidia-smi** — NVIDIA GPUs с VRAM
2. **lspci** — PCI перечисление
3. **sysfs** — `/sys/class/drm` для AMD/Intel

Результаты объединяются с приоритетом nvidia-smi для NVIDIA.
Для GPU процессов используются `nvidia-smi pmon` и `/proc/*/fdinfo` (DRM).

### Зависимости

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal handling
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — System information
- [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml) — Config parsing
- [thiserror](https://github.com/dtolnay/thiserror) — Error handling

### Лицензия

MIT
