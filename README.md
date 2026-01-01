# rtop

Терминальный монитор системы для Linux с поддержкой GPU.

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Linux-green)

## Возможности

- **Процессы** — CPU, память, аптайм, статус
- **GPU** — NVIDIA (nvidia-smi), AMD/Intel (sysfs/lspci)
- **VRAM** — использование памяти видеокарты в реальном времени
- **Дерево процессов** — раскрытие parent→child в Processes
- **Сортировка и фильтры** — быстрые клавиши + клики мышью по заголовкам
- **Системная вкладка** — расширенная информация + цветной ASCII‑логотип
- **Контейнеры** — список контейнеров и drill‑down в процессы
- **Setup/Help** — модальные окна (F2/F12)

## Установка

```bash
# Clone and build
git clone https://github.com/yourusername/rtop.git
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

## Использование

```bash
rtop [options]
```

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
| `Enter` | Завершить процесс (с подтверждением) |
| `c` / `m` / `p` / `n` / `u` | Быстрая сортировка CPU/Mem/PID/Name/User |
| `h` | Подсветка процессов (user/non‑root/GUI) |
| `g` / `G` | Следующий/предыдущий GPU |
| `t` | Дерево процессов (только в Processes) |
| `1` / `2` / `3` / `4` | Overview / System / GPU / Containers |
| `Tab` | Циклическое переключение вкладок |
| `b` / `Esc` | Назад из контейнерного drill‑down |
| `F2` | Setup |
| `F12` | Help |
| `r` | Принудительное обновление |

### Мышь

- ЛКМ по заголовку колонки — сортировка по колонке / смена направления.
- В режиме дерева сортировка фиксирована по PID.

## Конфигурация

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
```

CLI‑аргументы имеют приоритет над конфигом.

## Архитектура

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

## GPU детекция

rtop использует несколько источников:

1. **nvidia-smi** — NVIDIA GPUs с VRAM
2. **lspci** — PCI перечисление
3. **sysfs** — `/sys/class/drm` для AMD/Intel

Результаты объединяются с приоритетом nvidia-smi для NVIDIA.

## ASCII‑логотипы

Документация для добавления логотипов: `docs/ASCII_LOGOS.md`.

## Зависимости

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal handling
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — System information
- [serde](https://github.com/serde-rs/serde) + [toml](https://github.com/toml-rs/toml) — Config parsing
- [thiserror](https://github.com/dtolnay/thiserror) — Error handling

## Лицензия

MIT
