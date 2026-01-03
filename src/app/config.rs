use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

use super::state::Language;
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
    pub gpu_poll_rate: Duration,
    pub language: Language,
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
    language: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_vram: true,
            default_sort: "cpu".to_string(),
            sort_dir: String::new(),
            gpu_preference: "auto".to_string(),
            language: "en".to_string(),
        }
    }
}

impl Config {
    pub fn from_args() -> Result<Self, String> {
        // Load file config first
        let file_config = match load_config_file() {
            Ok(Some(config)) => config,
            Ok(None) => FileConfig::default(),
            Err(message) => {
                eprintln!("{message}");
                FileConfig::default()
            }
        };

        // Start with file config values
        let mut tick_ms = file_config.general.tick_rate_ms;
        let mut vram_enabled = file_config.display.show_vram;
        let mut gpu_poll_ms = file_config.general.gpu_poll_ms;
        let mut sort_key =
            SortKey::parse(&file_config.display.default_sort).unwrap_or(SortKey::Cpu);
        let mut sort_dir: Option<SortDir> = if file_config.display.sort_dir.is_empty() {
            None
        } else {
            SortDir::parse(&file_config.display.sort_dir)
        };
        let mut gpu_pref = GpuPreference::parse(&file_config.display.gpu_preference)
            .unwrap_or(GpuPreference::Auto);
        let language = Language::parse(&file_config.display.language).unwrap_or(Language::English);

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
        gpu_poll_ms = normalize_gpu_poll_ms(gpu_poll_ms);
        let sort_dir = sort_dir.unwrap_or_else(|| sort_key.default_dir());

        Ok(Self {
            tick_rate: Duration::from_millis(tick_ms),
            vram_enabled,
            sort_key,
            sort_dir,
            gpu_pref,
            gpu_poll_rate: Duration::from_millis(gpu_poll_ms),
            language,
        })
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("rtop").join("config.toml"))
}

fn load_config_file() -> Result<Option<FileConfig>, String> {
    let Some(path) = config_path() else {
        return Ok(None);
    };
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!(
                "Failed to read config file {}: {err}",
                path.display()
            ));
        }
    };
    match toml::from_str(&content) {
        Ok(config) => Ok(Some(config)),
        Err(err) => Err(format!(
            "Failed to parse config file {}: {err}",
            path.display()
        )),
    }
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
        "  --sort <key>       pid | user | cpu | mem | uptime | stat | name",
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
        "  language = \"en\"",
    ]
    .join("\n")
}

pub fn save_language_preference(language: Language) -> Result<(), String> {
    let Some(path) = config_path() else {
        return Err("Config path unavailable".to_string());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create config directory: {err}"))?;
    }

    let mut root = if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => content
                .parse::<toml::Value>()
                .map_err(|err| {
                    format!("Failed to parse config file {}: {err}", path.display())
                })?,
            Err(err) => {
                return Err(format!(
                    "Failed to read config file {}: {err}",
                    path.display()
                ));
            }
        }
    } else {
        toml::Value::Table(Default::default())
    };

    let table = root
        .as_table_mut()
        .ok_or_else(|| format!("Config file {} has invalid format", path.display()))?;
    let display = table
        .entry("display".to_string())
        .or_insert_with(|| toml::Value::Table(Default::default()));
    let display_table = display
        .as_table_mut()
        .ok_or_else(|| format!("Config file {} has invalid [display] section", path.display()))?;
    display_table.insert(
        "language".to_string(),
        toml::Value::String(language.code().to_string()),
    );

    let output = toml::to_string_pretty(&root)
        .map_err(|err| format!("Failed to serialize config: {err}"))?;
    fs::write(&path, output)
        .map_err(|err| format!("Failed to write config file {}: {err}", path.display()))?;
    Ok(())
}

fn normalize_tick_ms(value: u64) -> u64 {
    value.max(MIN_TICK_MS)
}

fn normalize_gpu_poll_ms(value: u64) -> u64 {
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
        assert_eq!(config.display.language, "en");
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
        assert_eq!(config.display.language, "en");
    }
}
