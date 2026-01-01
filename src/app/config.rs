use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

use crate::data::{GpuPreference, SortDir, SortKey};

const MIN_TICK_MS: u64 = 100;
const DEFAULT_TICK_MS: u64 = 1000;

/// Runtime configuration
pub struct Config {
    pub tick_rate: Duration,
    pub vram_enabled: bool,
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
    pub gpu_pref: GpuPreference,
}

/// File-based configuration (TOML)
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileConfig {
    general: GeneralConfig,
    display: DisplayConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct GeneralConfig {
    tick_rate_ms: u64,
    gpu_poll_ms: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: DEFAULT_TICK_MS,
            gpu_poll_ms: 2000,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct DisplayConfig {
    show_vram: bool,
    default_sort: String,
    sort_dir: String,
    gpu_preference: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_vram: true,
            default_sort: "cpu".to_string(),
            sort_dir: String::new(),
            gpu_preference: "auto".to_string(),
        }
    }
}

impl Config {
    pub fn from_args() -> Result<Self, String> {
        // Load file config first
        let file_config = load_config_file().unwrap_or_default();

        // Start with file config values
        let mut tick_ms = file_config.general.tick_rate_ms;
        let mut vram_enabled = file_config.display.show_vram;
        let mut sort_key =
            SortKey::parse(&file_config.display.default_sort).unwrap_or(SortKey::Cpu);
        let mut sort_dir: Option<SortDir> = if file_config.display.sort_dir.is_empty() {
            None
        } else {
            SortDir::parse(&file_config.display.sort_dir)
        };
        let mut gpu_pref = GpuPreference::parse(&file_config.display.gpu_preference)
            .unwrap_or(GpuPreference::Auto);

        // Override with CLI args
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => return Err(usage()),
                "--tick-ms" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "Missing value for --tick-ms\n\n".to_string() + &usage())?;
                    tick_ms = value
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid tick value: {value}\n\n{}", usage()))?;
                }
                "--no-vram" => vram_enabled = false,
                "--sort" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "Missing value for --sort\n\n".to_string() + &usage())?;
                    sort_key = SortKey::parse(&value)
                        .ok_or_else(|| format!("Invalid sort key: {value}\n\n{}", usage()))?;
                }
                "--sort-dir" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "Missing value for --sort-dir\n\n".to_string() + &usage())?;
                    sort_dir = Some(
                        SortDir::parse(&value)
                            .ok_or_else(|| format!("Invalid sort dir: {value}\n\n{}", usage()))?,
                    );
                }
                "--gpu" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "Missing value for --gpu\n\n".to_string() + &usage())?;
                    gpu_pref = GpuPreference::parse(&value)
                        .ok_or_else(|| format!("Invalid gpu preference: {value}\n\n{}", usage()))?;
                }
                _ => return Err(format!("Unknown argument: {arg}\n\n{}", usage())),
            }
        }

        tick_ms = normalize_tick_ms(tick_ms);
        let sort_dir = sort_dir.unwrap_or_else(|| sort_key.default_dir());

        Ok(Self {
            tick_rate: Duration::from_millis(tick_ms),
            vram_enabled,
            sort_key,
            sort_dir,
            gpu_pref,
        })
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("rtop").join("config.toml"))
}

fn load_config_file() -> Option<FileConfig> {
    let path = config_path()?;
    let content = fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

fn usage() -> String {
    let config_location = config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/rtop/config.toml".to_string());

    [
        "Usage: rtop [options]",
        "",
        "Options:",
        "  --tick-ms <ms>     Refresh interval in milliseconds (default: 1000, min: 100)",
        "  --no-vram          Disable GPU probing",
        "  --sort <key>       pid | cpu | mem | uptime | stat | name",
        "  --sort-dir <dir>   asc | desc",
        "  --gpu <pref>       auto | discrete | integrated",
        "  -h, --help         Show this help",
        "",
        &format!("Config file: {config_location}"),
        "",
        "Example config.toml:",
        "  [general]",
        "  tick_rate_ms = 1000",
        "  gpu_poll_ms = 2000",
        "",
        "  [display]",
        "  show_vram = true",
        "  default_sort = \"cpu\"",
        "  sort_dir = \"desc\"",
        "  gpu_preference = \"auto\"",
    ]
    .join("\n")
}

fn normalize_tick_ms(value: u64) -> u64 {
    value.max(MIN_TICK_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tick_ms_clamps_to_min() {
        assert_eq!(normalize_tick_ms(0), MIN_TICK_MS);
        assert_eq!(normalize_tick_ms(MIN_TICK_MS), MIN_TICK_MS);
        assert_eq!(normalize_tick_ms(MIN_TICK_MS + 5), MIN_TICK_MS + 5);
    }

    #[test]
    fn file_config_defaults() {
        let config: FileConfig = toml::from_str("").unwrap();
        assert_eq!(config.general.tick_rate_ms, DEFAULT_TICK_MS);
        assert!(config.display.show_vram);
        assert_eq!(config.display.default_sort, "cpu");
    }

    #[test]
    fn file_config_partial() {
        let config: FileConfig = toml::from_str(
            r#"
            [display]
            default_sort = "mem"
            "#,
        )
        .unwrap();
        assert_eq!(config.general.tick_rate_ms, DEFAULT_TICK_MS);
        assert_eq!(config.display.default_sort, "mem");
    }
}
