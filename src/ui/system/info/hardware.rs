use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use sysinfo::Motherboard;

use crate::app::{App, Language};
use crate::data::gpu::{GpuKind, gpu_vendor_label, short_device_name};
use crate::ui::text::tr;
use crate::utils::{format_bytes, percent, run_command_with_timeout};

pub fn motherboard_summary() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let board = Motherboard::new()?;
            let name = board.name().filter(|value| !value.trim().is_empty());
            let vendor = board.vendor_name().filter(|value| !value.trim().is_empty());
            let version = board.version().filter(|value| !value.trim().is_empty());
            let mut line = name.or(vendor)?;
            if let Some(version) = version
                && !line.contains(&version)
            {
                line = format!("{line} ({version})");
            }
            Some(line)
        })
        .clone()
}

pub fn cpu_overview_line(cpu_brand: &str, cpu_count: usize, cpu_list: &[sysinfo::Cpu]) -> String {
    let mut line = format!("{cpu_brand} ({cpu_count})");
    if let Some((_min_mhz, max_mhz)) = cpu_passport_freq_range() {
        line.push_str(" @ ");
        line.push_str(&format_freq(max_mhz));
    } else if let Some(freq) = max_cpu_freq(cpu_list) {
        line.push_str(" @ ");
        line.push_str(&freq);
    }
    line
}

pub fn summarize_cpu_freq(cpus: &[sysinfo::Cpu]) -> Option<String> {
    let freqs = cpus
        .iter()
        .map(|cpu| cpu.frequency())
        .filter(|freq| *freq > 0)
        .collect::<Vec<_>>();
    if freqs.is_empty() {
        return None;
    }
    let total: u64 = freqs.iter().sum();
    let avg = total / freqs.len() as u64;
    Some(format_freq(avg))
}

fn cpu_passport_freq_range() -> Option<(u64, u64)> {
    static CACHE: OnceLock<Option<(u64, u64)>> = OnceLock::new();
    *CACHE.get_or_init(read_cpu_passport_freq_range)
}

fn read_cpu_passport_freq_range() -> Option<(u64, u64)> {
    let base = Path::new("/sys/devices/system/cpu/cpufreq");
    let entries = fs::read_dir(base).ok()?;
    let mut min_khz: Option<u64> = None;
    let mut max_khz: Option<u64> = None;
    let mut base_khz: Option<u64> = None;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("policy") {
            continue;
        }
        let path = entry.path();
        if let Some(value) = read_khz(path.join("cpuinfo_min_freq")) {
            min_khz = Some(min_khz.map_or(value, |current| current.min(value)));
        }
        if let Some(value) = read_khz(path.join("cpuinfo_max_freq")) {
            max_khz = Some(max_khz.map_or(value, |current| current.max(value)));
        }
        if let Some(value) = read_khz(path.join("base_frequency")) {
            base_khz = Some(base_khz.map_or(value, |current| current.min(value)));
        }
    }

    let max_khz = max_khz?;
    let min_khz = base_khz.or(min_khz).unwrap_or(max_khz);
    Some((min_khz / 1000, max_khz / 1000))
}

fn read_khz(path: PathBuf) -> Option<u64> {
    let content = fs::read_to_string(path).ok()?;
    let value = content.split_whitespace().next()?;
    let parsed = value.parse::<u64>().ok()?;
    if parsed == 0 { None } else { Some(parsed) }
}

fn max_cpu_freq(cpus: &[sysinfo::Cpu]) -> Option<String> {
    let max = cpus
        .iter()
        .map(|cpu| cpu.frequency())
        .filter(|freq| *freq > 0)
        .max()?;
    Some(format_freq(max))
}

pub fn format_freq(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{mhz} MHz")
    }
}

pub fn gpu_summary(app: &App, language: Language) -> Option<String> {
    let (_idx, gpu) = app.selected_gpu()?;
    let vendor = gpu_vendor_label(gpu);
    let device_name = gpu
        .device
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&gpu.name);
    let short_name = short_device_name(device_name);
    let mut label = if short_name.is_empty() {
        vendor
    } else {
        format!("{vendor} {short_name}")
    };
    let kind_label = match gpu.kind {
        GpuKind::Discrete => Some(tr(language, "Discrete", "Дискретная")),
        GpuKind::Integrated => Some(tr(language, "Integrated", "Встроенная")),
        GpuKind::Unknown => None,
    };
    if let Some(kind_label) = kind_label {
        label.push_str(" [");
        label.push_str(kind_label);
        label.push(']');
    }
    Some(label)
}

pub fn disk_summary_lines(app: &App) -> Vec<String> {
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for disk in app.disks.iter() {
        let total = disk.total_space();
        if total == 0 {
            continue;
        }
        let fs = disk.file_system().to_string_lossy();
        if should_skip_fs(&fs) {
            continue;
        }
        let avail = disk.available_space();
        let used = total.saturating_sub(avail);
        let pct = percent(used, total);
        let mut line = format!(
            "{} / {} ({pct:.0}%)",
            format_bytes(used),
            format_bytes(total)
        );
        if !fs.trim().is_empty() {
            line.push_str(" - ");
            line.push_str(fs.trim());
        }
        if seen.insert(line.clone()) {
            entries.push((total, line));
        }
    }
    entries.sort_by(|(a_total, _), (b_total, _)| b_total.cmp(a_total));
    entries.into_iter().map(|(_, line)| line).collect()
}

fn should_skip_fs(fs: &str) -> bool {
    matches!(
        fs,
        "tmpfs"
            | "devtmpfs"
            | "overlay"
            | "squashfs"
            | "proc"
            | "sysfs"
            | "cgroup2"
            | "debugfs"
            | "tracefs"
            | "configfs"
            | "mqueue"
            | "hugetlbfs"
            | "ramfs"
            | "autofs"
            | "fusectl"
            | "pstore"
            | "securityfs"
            | "selinuxfs"
            | "binfmt_misc"
    )
}

#[derive(Clone, Copy)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: Option<f32>,
    pub size_in: Option<f32>,
    pub is_external: Option<bool>,
}

pub fn display_summary(language: Language) -> Option<String> {
    static CACHE: OnceLock<Option<DisplayInfo>> = OnceLock::new();
    let info = CACHE.get_or_init(display_info);
    let info = info.as_ref()?;
    Some(format_display_info(info, language))
}

fn display_info() -> Option<DisplayInfo> {
    display_from_xrandr().or_else(display_from_drm)
}

fn display_from_xrandr() -> Option<DisplayInfo> {
    let output = run_command_with_timeout("xrandr", &["--query"], Duration::from_millis(400))?;
    let mut lines = output.lines().peekable();
    let mut fallback = None;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with(' ') || !line.contains(" connected") {
            continue;
        }
        let is_primary = line.contains(" primary ");
        let connector = line.split_whitespace().next().unwrap_or_default();
        let (width, height) = parse_xrandr_resolution(line)?;
        let size_in = parse_xrandr_size(line).and_then(|(w_mm, h_mm)| mm_to_inches(w_mm, h_mm));
        let refresh_hz = parse_xrandr_refresh(&mut lines);
        let info = DisplayInfo {
            width,
            height,
            refresh_hz,
            size_in,
            is_external: Some(is_external_connector(connector)),
        };
        if is_primary {
            return Some(info);
        }
        if fallback.is_none() {
            fallback = Some(info);
        }
    }
    fallback
}

fn parse_xrandr_resolution(line: &str) -> Option<(u32, u32)> {
    for token in line.split_whitespace() {
        if !token.contains('x') {
            continue;
        }
        if !token
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            continue;
        }
        let token = token.split('+').next().unwrap_or(token);
        let (w, h) = token.split_once('x')?;
        let width = w.parse::<u32>().ok()?;
        let height = h.parse::<u32>().ok()?;
        return Some((width, height));
    }
    None
}

fn parse_xrandr_size(line: &str) -> Option<(u32, u32)> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    for window in tokens.windows(3) {
        if window[1] != "x" {
            continue;
        }
        if !window[0].ends_with("mm") || !window[2].ends_with("mm") {
            continue;
        }
        let w = window[0].trim_end_matches("mm").parse::<u32>().ok()?;
        let h = window[2].trim_end_matches("mm").parse::<u32>().ok()?;
        return Some((w, h));
    }
    None
}

fn parse_xrandr_refresh(lines: &mut std::iter::Peekable<std::str::Lines<'_>>) -> Option<f32> {
    let mut refresh = None;
    while let Some(line) = lines.peek() {
        if !line.starts_with(' ') {
            break;
        }
        let line = lines.next().unwrap();
        if let Some(value) = parse_refresh_token(line) {
            refresh = Some(value);
        }
    }
    refresh
}

fn parse_refresh_token(line: &str) -> Option<f32> {
    for token in line.split_whitespace() {
        if !token.contains('*') {
            continue;
        }
        let mut value = String::new();
        let mut started = false;
        for ch in token.chars() {
            if ch.is_ascii_digit() {
                started = true;
                value.push(ch);
            } else if started && ch == '.' {
                value.push(ch);
            } else if started {
                break;
            }
        }
        if !value.is_empty() {
            return value.parse::<f32>().ok();
        }
    }
    None
}

fn display_from_drm() -> Option<DisplayInfo> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.contains('-') {
            continue;
        }
        let path = entry.path();
        let status = fs::read_to_string(path.join("status")).ok()?;
        if status.trim() != "connected" {
            continue;
        }
        let mode = fs::read_to_string(path.join("modes"))
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .map(str::to_string)
            })?;
        let (width, height) = parse_mode_resolution(&mode)?;
        let size_in = fs::read(path.join("edid"))
            .ok()
            .and_then(|data| edid_size_inches(&data));
        let connector = name.split_once('-').map(|x| x.1).unwrap_or(name.as_str());
        return Some(DisplayInfo {
            width,
            height,
            refresh_hz: None,
            size_in,
            is_external: Some(is_external_connector(connector)),
        });
    }
    None
}

fn parse_mode_resolution(mode: &str) -> Option<(u32, u32)> {
    let (w, h) = mode.trim().split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn mm_to_inches(width_mm: u32, height_mm: u32) -> Option<f32> {
    if width_mm == 0 || height_mm == 0 {
        return None;
    }
    let diag_mm = ((width_mm * width_mm + height_mm * height_mm) as f32).sqrt();
    Some(diag_mm / 25.4)
}

fn edid_size_inches(edid: &[u8]) -> Option<f32> {
    if edid.len() < 23 {
        return None;
    }
    let width_cm = edid[21] as f32;
    let height_cm = edid[22] as f32;
    if width_cm == 0.0 || height_cm == 0.0 {
        return None;
    }
    let diag_cm = (width_cm * width_cm + height_cm * height_cm).sqrt();
    Some(diag_cm / 2.54)
}

fn format_display_info(info: &DisplayInfo, language: Language) -> String {
    let mut line = format!("{}x{}", info.width, info.height);
    if let Some(refresh) = info.refresh_hz {
        line.push_str(&format!(" @ {:.0} Hz", refresh));
    }
    if let Some(size) = info.size_in {
        line.push_str(&format!(" in {:.0}\"", size));
    }
    if let Some(is_external) = info.is_external {
        let label = if is_external {
            tr(language, "External", "Внешний")
        } else {
            tr(language, "Internal", "Встроенный")
        };
        line.push_str(" [");
        line.push_str(label);
        line.push(']');
    }
    line
}

fn is_external_connector(connector: &str) -> bool {
    let lower = connector.to_ascii_lowercase();
    !(lower.contains("edp") || lower.contains("lvds") || lower.contains("dsi"))
}

pub fn mouse_name() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(mouse_name_inner).clone()
}

fn mouse_name_inner() -> Option<String> {
    let content = fs::read_to_string("/proc/bus/input/devices").ok()?;
    let mut candidates = Vec::new();
    for block in content.split("\n\n") {
        let mut name = None;
        let mut handlers = None;
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("N: Name=") {
                name = Some(value.trim().trim_matches('"').to_string());
            } else if let Some(value) = line.strip_prefix("H: Handlers=") {
                handlers = Some(value.to_string());
            }
        }
        if let (Some(name), Some(handlers)) = (name, handlers)
            && handlers.contains("mouse")
        {
            candidates.push(name);
        }
    }
    choose_mouse(candidates)
}

fn choose_mouse(candidates: Vec<String>) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    let mut best = None;
    for candidate in candidates {
        if !is_touchpad_name(&candidate) {
            return Some(candidate);
        }
        if best.is_none() {
            best = Some(candidate);
        }
    }
    best
}

fn is_touchpad_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("touchpad") || lower.contains("trackpad") || lower.contains("trackpoint")
}
