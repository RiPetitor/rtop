use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use sysinfo::{Motherboard, Pid, System};

use crate::app::{App, Language, SystemOverviewSnapshot, SystemTab};
use crate::data::gpu::{GpuKind, gpu_vendor_label, short_device_name};
use crate::ui::text::tr;
use crate::ui::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::utils::{fit_text, format_bytes, percent, run_command_with_timeout, text_width};

pub fn render_info(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    render_info_body(frame, area, app);
}

fn render_info_body(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let section_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);
    let icon_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let sep_style = Style::default().fg(COLOR_MUTED);
    let width = area.width.max(1) as usize;
    let label_width = width.min(12).max(6).min(width);

    let unknown = tr(app.language, "unknown", "неизвестно");
    let na = tr(app.language, "n/a", "н/д");

    if app.system_tab == SystemTab::Overview && app.system_overview_snapshot.is_none() {
        let snapshot = build_system_overview_snapshot(app);
        app.system_overview_snapshot = Some(snapshot);
    }

    let cpu_list = app.system.cpus();
    let cpu_brand = cpu_list
        .first()
        .map(|cpu| cpu.brand().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| unknown.to_string());
    let cpu_count = cpu_list.len();
    let cpu_physical = System::physical_core_count();
    let cpu_cores = match cpu_physical {
        Some(physical) if physical > 0 => format!("{physical}P / {cpu_count}L"),
        _ => format!("{cpu_count}L"),
    };
    let cpu_freq = summarize_cpu_freq(cpu_list).unwrap_or_else(|| na.to_string());
    let cpu_usage = app.system.global_cpu_usage();
    let load = System::load_average();
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let free_mem = app.system.free_memory();
    let avail_mem = app.system.available_memory();
    let mem_pct = percent(used_mem, total_mem);
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let swap_pct = percent(used_swap, total_swap);
    let processes = app.system.processes().len();
    let net_refresh = app.network_refresh_secs.filter(|value| *value > 0.0);

    let mut lines = Vec::new();
    let push_main = |lines: &mut Vec<Line<'static>>, snapshot: &SystemOverviewSnapshot| {
        push_icon_line(
            lines,
            ICON_USER,
            snapshot.user_host.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            ICON_DISTRO,
            snapshot.distro_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_OS,
            snapshot.os_name.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_KERNEL,
            snapshot.kernel_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_UPTIME,
            snapshot.uptime_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            ICON_BOARD,
            snapshot.board_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_CPU,
            snapshot.cpu_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_GPU,
            snapshot.gpu_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_MEM,
            snapshot.mem_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );

        if snapshot.disk_lines.is_empty() {
            push_icon_line(
                lines,
                ICON_DISK,
                na.to_string(),
                width,
                icon_style,
                sep_style,
                value_style,
            );
        } else {
            for value in snapshot.disk_lines.iter().cloned() {
                push_icon_line(
                    lines,
                    ICON_DISK,
                    value,
                    width,
                    icon_style,
                    sep_style,
                    value_style,
                );
            }
        }

        push_icon_line(
            lines,
            ICON_DISPLAY,
            snapshot.display_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_MOUSE,
            snapshot.mouse_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            ICON_DE,
            snapshot.de_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_WM,
            snapshot.wm_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_SHELL,
            snapshot.shell_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_TERM,
            snapshot.terminal_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
        push_icon_line(
            lines,
            ICON_PKG,
            snapshot.package_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
        );
    };
    let push_cpu = |lines: &mut Vec<Line<'static>>| {
        push_header(lines, tr(app.language, "CPU", "CPU"), width, section_style);
        push_line(
            lines,
            tr(app.language, "Model", "Модель"),
            cpu_brand.clone(),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Cores", "Ядра"),
            cpu_cores.clone(),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Freq", "Част."),
            cpu_freq.clone(),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Usage", "Загр."),
            format!("{cpu_usage:.1}%"),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Load", "Нагрузка"),
            format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
            width,
            label_width,
            label_style,
            value_style,
        );
    };
    let push_memory = |lines: &mut Vec<Line<'static>>| {
        push_header(
            lines,
            tr(app.language, "Memory", "Память"),
            width,
            section_style,
        );
        push_line(
            lines,
            tr(app.language, "RAM", "ОЗУ"),
            format!(
                "{} / {} ({mem_pct:.0}%)",
                format_bytes(used_mem),
                format_bytes(total_mem)
            ),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Avail", "Дост."),
            format_bytes(avail_mem),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Free", "Своб."),
            format_bytes(free_mem),
            width,
            label_width,
            label_style,
            value_style,
        );
        push_line(
            lines,
            tr(app.language, "Swap", "Swap"),
            format!(
                "{} / {} ({swap_pct:.0}%)",
                format_bytes(used_swap),
                format_bytes(total_swap)
            ),
            width,
            label_width,
            label_style,
            value_style,
        );
    };
    let _push_processes = |lines: &mut Vec<Line<'static>>| {
        push_header(
            lines,
            tr(app.language, "Processes", "Процессы"),
            width,
            section_style,
        );
        push_line(
            lines,
            tr(app.language, "Total", "Всего"),
            processes.to_string(),
            width,
            label_width,
            label_style,
            value_style,
        );
    };
    let push_disks = |lines: &mut Vec<Line<'static>>| {
        push_header(
            lines,
            tr(app.language, "Disks", "Диски"),
            width,
            section_style,
        );
        if app.disks.is_empty() {
            push_line(
                lines,
                tr(app.language, "Disk", "Диск"),
                na.to_string(),
                width,
                label_width,
                label_style,
                value_style,
            );
            return;
        }
        for disk in app.disks.iter() {
            let total = disk.total_space();
            let avail = disk.available_space();
            let used = total.saturating_sub(avail);
            let pct = percent(used, total);
            let mount = disk.mount_point().display().to_string();
            let fs = disk.file_system().to_string_lossy();
            let value = format!(
                "{} / {} ({pct:.0}%) {fs}",
                format_bytes(used),
                format_bytes(total)
            );
            push_line(
                lines,
                &mount,
                value,
                width,
                label_width,
                label_style,
                value_style,
            );
        }
    };
    let push_network = |lines: &mut Vec<Line<'static>>| {
        push_header(
            lines,
            tr(app.language, "Network", "Сеть"),
            width,
            section_style,
        );
        if app.networks.is_empty() {
            push_line(
                lines,
                tr(app.language, "Net", "Сеть"),
                na.to_string(),
                width,
                label_width,
                label_style,
                value_style,
            );
            return;
        }
        let mut networks = app.networks.iter().collect::<Vec<_>>();
        networks.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (name, data) in networks {
            let value = if let Some(secs) = net_refresh {
                let rx_rate = (data.received() as f64 / secs).round() as u64;
                let tx_rate = (data.transmitted() as f64 / secs).round() as u64;
                format!(
                    "rx {}/s tx {}/s",
                    format_bytes(rx_rate),
                    format_bytes(tx_rate)
                )
            } else {
                format!(
                    "rx {} tx {}",
                    format_bytes(data.total_received()),
                    format_bytes(data.total_transmitted())
                )
            };
            push_line(
                lines,
                name,
                value,
                width,
                label_width,
                label_style,
                value_style,
            );
        }
    };
    let push_temps = |lines: &mut Vec<Line<'static>>| {
        push_header(
            lines,
            tr(app.language, "Temps", "Темп."),
            width,
            section_style,
        );
        if app.components.is_empty() {
            push_line(
                lines,
                tr(app.language, "Temp", "Темп."),
                na.to_string(),
                width,
                label_width,
                label_style,
                value_style,
            );
            return;
        }
        let mut temps = app
            .components
            .iter()
            .map(|component| (component.label().to_string(), component.temperature()))
            .collect::<Vec<_>>();
        temps.sort_by(
            |(a_label, a_temp), (b_label, b_temp)| match (a_temp, b_temp) {
                (Some(a_temp), Some(b_temp)) => b_temp
                    .partial_cmp(a_temp)
                    .unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a_label.cmp(b_label),
            },
        );
        for (label, temp) in temps {
            let value = temp
                .map(|value| format!("{value:.1}°C"))
                .unwrap_or_else(|| na.to_string());
            push_line(
                lines,
                &label,
                value,
                width,
                label_width,
                label_style,
                value_style,
            );
        }
    };

    match app.system_tab {
        SystemTab::Overview => {
            if let Some(snapshot) = app.system_overview_snapshot.as_ref() {
                push_main(&mut lines, snapshot);
            }
        }
        SystemTab::Cpu => push_cpu(&mut lines),
        SystemTab::Memory => push_memory(&mut lines),
        SystemTab::Disks => push_disks(&mut lines),
        SystemTab::Network => push_network(&mut lines),
        SystemTab::Temps => push_temps(&mut lines),
    }

    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn push_header(lines: &mut Vec<Line<'static>>, title: &str, width: usize, style: Style) {
    let title = fit_text(title, width);
    lines.push(Line::from(Span::styled(title, style)));
}

fn push_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: String,
    width: usize,
    label_width: usize,
    label_style: Style,
    value_style: Style,
) {
    let label = pad_label(label, label_width);
    let max_value = width.saturating_sub(text_width(&label)).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(label, label_style),
        Span::styled(value, value_style),
    ]));
}

fn pad_label(label: &str, width: usize) -> String {
    let trimmed = fit_text(label, width);
    let pad = width.saturating_sub(text_width(&trimmed));
    if pad == 0 {
        trimmed
    } else {
        format!("{trimmed}{}", " ".repeat(pad))
    }
}

fn summarize_cpu_freq(cpus: &[sysinfo::Cpu]) -> Option<String> {
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
    let value = content.trim().split_whitespace().next()?;
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

fn format_freq(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f64 / 1000.0)
    } else {
        format!("{mhz} MHz")
    }
}

const ICON_USER: &str = "";
const ICON_DISTRO: &str = "󱋩";
const ICON_OS: &str = "󰣛";
const ICON_KERNEL: &str = "";
const ICON_UPTIME: &str = "󰅐";
const ICON_BOARD: &str = "󰾰";
const ICON_CPU: &str = "󰻠";
const ICON_GPU: &str = "󰍛";
const ICON_MEM: &str = "";
const ICON_DISK: &str = "";
const ICON_DISPLAY: &str = "󰍹";
const ICON_MOUSE: &str = "󰖺";
const ICON_DE: &str = "󰕮";
const ICON_WM: &str = "";
const ICON_SHELL: &str = "";
const ICON_TERM: &str = "";
const ICON_PKG: &str = "󰏖";
const ICON_SEP: &str = "";
const ICON_IMMUTABLE: &str = "";

#[derive(Default, Clone)]
struct OsRelease {
    name: Option<String>,
    pretty_name: Option<String>,
    id: Option<String>,
    version: Option<String>,
    version_id: Option<String>,
    variant: Option<String>,
    variant_id: Option<String>,
    image_id: Option<String>,
    build_id: Option<String>,
}

#[derive(Clone, Copy)]
struct DisplayInfo {
    width: u32,
    height: u32,
    refresh_hz: Option<f32>,
    size_in: Option<f32>,
    is_external: Option<bool>,
}

fn os_release() -> OsRelease {
    static CACHE: OnceLock<OsRelease> = OnceLock::new();
    CACHE.get_or_init(load_os_release).clone()
}

fn load_os_release() -> OsRelease {
    let content = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    parse_os_release(&content)
}

fn parse_os_release(content: &str) -> OsRelease {
    let mut info = OsRelease::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key {
            "NAME" => info.name = Some(value),
            "PRETTY_NAME" => info.pretty_name = Some(value),
            "ID" => info.id = Some(value),
            "VERSION" => info.version = Some(value),
            "VERSION_ID" => info.version_id = Some(value),
            "VARIANT" => info.variant = Some(value),
            "VARIANT_ID" => info.variant_id = Some(value),
            "IMAGE_ID" => info.image_id = Some(value),
            "BUILD_ID" => info.build_id = Some(value),
            _ => {}
        }
    }
    info
}

fn distro_variant_line(info: &OsRelease) -> Option<String> {
    let id = info
        .image_id
        .as_ref()
        .or(info.id.as_ref())
        .or(info.name.as_ref())?;
    let mut line = id.clone();
    if let Some(variant) = info.variant_id.as_ref().or(info.variant.as_ref()) {
        line.push(':');
        line.push_str(variant);
    } else if let Some(version) = info.version_id.as_ref() {
        line.push(':');
        line.push_str(version);
    }
    if is_immutable_os() {
        line.push(' ');
        line.push_str(ICON_IMMUTABLE);
    }
    Some(line)
}

fn is_immutable_os() -> bool {
    Path::new("/run/ostree-booted").exists() || Path::new("/sysroot/ostree").exists()
}

fn push_icon_line(
    lines: &mut Vec<Line<'static>>,
    icon: &str,
    value: String,
    width: usize,
    icon_style: Style,
    sep_style: Style,
    value_style: Style,
) {
    let icon_text = format!("{icon} ");
    let sep_text = format!("{ICON_SEP} ");
    let used = text_width(&icon_text) + text_width(&sep_text);
    let max_value = width.saturating_sub(used).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(icon_text, icon_style),
        Span::styled(sep_text, sep_style),
        Span::styled(value, value_style),
    ]));
}

fn format_uptime_long(uptime_secs: u64, language: Language) -> String {
    let mut remaining = uptime_secs;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let minutes = remaining / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!(
            "{days} {}",
            pluralize(language, days, "day", "days", "день", "дня", "дней")
        ));
    }
    if hours > 0 {
        parts.push(format!(
            "{hours} {}",
            pluralize(language, hours, "hour", "hours", "час", "часа", "часов")
        ));
    }
    if minutes > 0 || parts.is_empty() {
        parts.push(format!(
            "{minutes} {}",
            pluralize(
                language,
                minutes,
                "min",
                "mins",
                "минута",
                "минуты",
                "минут"
            )
        ));
    }
    parts.join(", ")
}

fn pluralize<'a>(
    language: Language,
    value: u64,
    en_one: &'a str,
    en_many: &'a str,
    ru_one: &'a str,
    ru_few: &'a str,
    ru_many: &'a str,
) -> &'a str {
    match language {
        Language::English => {
            if value == 1 {
                en_one
            } else {
                en_many
            }
        }
        Language::Russian => {
            let mod10 = value % 10;
            let mod100 = value % 100;
            if mod10 == 1 && mod100 != 11 {
                ru_one
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                ru_few
            } else {
                ru_many
            }
        }
    }
}

fn motherboard_summary() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let board = Motherboard::new()?;
            let name = board.name().filter(|value| !value.trim().is_empty());
            let vendor = board.vendor_name().filter(|value| !value.trim().is_empty());
            let version = board.version().filter(|value| !value.trim().is_empty());
            let mut line = name.or(vendor)?;
            if let Some(version) = version {
                if !line.contains(&version) {
                    line = format!("{line} ({version})");
                }
            }
            Some(line)
        })
        .clone()
}

fn cpu_overview_line(cpu_brand: &str, cpu_count: usize, cpu_list: &[sysinfo::Cpu]) -> String {
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

fn build_system_overview_snapshot(app: &App) -> SystemOverviewSnapshot {
    let unknown = tr(app.language, "unknown", "неизвестно");
    let na = tr(app.language, "n/a", "н/д");

    let host = System::host_name().unwrap_or_else(|| unknown.to_string());
    let user = app.current_user_name().unwrap_or(unknown);
    let user_host = format!("{user}@{host}");

    let os_release = os_release();
    let distro_line = distro_variant_line(&os_release).unwrap_or_else(|| na.to_string());
    let os_name = os_release
        .name
        .as_ref()
        .or(os_release.pretty_name.as_ref())
        .cloned()
        .or_else(System::name)
        .unwrap_or_else(|| unknown.to_string());

    let kernel = System::kernel_version().unwrap_or_else(|| unknown.to_string());
    let kernel_line = format!("Linux {kernel}");
    let uptime_line = format_uptime_long(System::uptime(), app.language);

    let board_line = motherboard_summary().unwrap_or_else(|| na.to_string());

    let cpu_list = app.system.cpus();
    let cpu_brand = cpu_list
        .first()
        .map(|cpu| cpu.brand().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| unknown.to_string());
    let cpu_count = cpu_list.len();
    let cpu_line = cpu_overview_line(&cpu_brand, cpu_count, cpu_list);

    let gpu_line = gpu_summary(app, app.language).unwrap_or_else(|| na.to_string());

    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let mem_pct = percent(used_mem, total_mem);
    let mem_line = format!(
        "{} / {} ({mem_pct:.0}%)",
        format_bytes(used_mem),
        format_bytes(total_mem)
    );

    let mut disk_lines = disk_summary_lines(app);
    if disk_lines.is_empty() {
        disk_lines.push(na.to_string());
    }

    let display_line = display_summary(app.language).unwrap_or_else(|| na.to_string());
    let mouse_line = mouse_name().unwrap_or_else(|| na.to_string());
    let de_line = desktop_environment().unwrap_or_else(|| na.to_string());
    let wm_line = window_manager(app).unwrap_or_else(|| na.to_string());
    let shell_line = shell_name().unwrap_or_else(|| na.to_string());
    let terminal_line = terminal_name(app).unwrap_or_else(|| na.to_string());
    let package_line = package_summary().unwrap_or_else(|| na.to_string());

    SystemOverviewSnapshot {
        user_host,
        distro_line,
        os_name,
        kernel_line,
        uptime_line,
        board_line,
        cpu_line,
        gpu_line,
        mem_line,
        disk_lines,
        display_line,
        mouse_line,
        de_line,
        wm_line,
        shell_line,
        terminal_line,
        package_line,
    }
}

fn gpu_summary(app: &App, language: Language) -> Option<String> {
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

fn disk_summary_lines(app: &App) -> Vec<String> {
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

fn display_summary(language: Language) -> Option<String> {
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
        let connector = name.splitn(2, '-').nth(1).unwrap_or(name.as_str());
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

fn mouse_name() -> Option<String> {
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
        if let (Some(name), Some(handlers)) = (name, handlers) {
            if handlers.contains("mouse") {
                candidates.push(name);
            }
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

fn desktop_environment() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(desktop_environment_inner).clone()
}

fn desktop_environment_inner() -> Option<String> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("XDG_SESSION_DESKTOP"))
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .ok()?;
    let primary = desktop.split(':').next().unwrap_or(desktop.as_str());
    let lower = primary.to_ascii_lowercase();

    if lower.contains("kde") || lower.contains("plasma") {
        let version = command_version("plasmashell", &["--version"]);
        return Some(match version {
            Some(ver) => format!("KDE Plasma {ver}"),
            None => "KDE Plasma".to_string(),
        });
    }
    if lower.contains("gnome") {
        let version = command_version("gnome-shell", &["--version"]);
        return Some(match version {
            Some(ver) => format!("GNOME {ver}"),
            None => "GNOME".to_string(),
        });
    }
    if lower.contains("xfce") {
        return Some("XFCE".to_string());
    }
    if lower.contains("lxqt") {
        return Some("LXQt".to_string());
    }
    if lower.contains("lxde") {
        return Some("LXDE".to_string());
    }
    if lower.contains("cinnamon") {
        return Some("Cinnamon".to_string());
    }
    if lower.contains("mate") {
        return Some("MATE".to_string());
    }
    if lower.contains("budgie") {
        return Some("Budgie".to_string());
    }
    if lower.contains("deepin") {
        return Some("Deepin".to_string());
    }

    Some(primary.to_string())
}

fn window_manager(app: &App) -> Option<String> {
    let mut wm = None;
    for process in app.system.processes().values() {
        let name = process.name().to_string_lossy().to_ascii_lowercase();
        let detected = match name.as_str() {
            "kwin_wayland" | "kwin_x11" | "kwin" => Some("KWin"),
            "mutter" | "gnome-shell" => Some("Mutter"),
            "sway" => Some("Sway"),
            "hyprland" => Some("Hyprland"),
            "wayfire" => Some("Wayfire"),
            "river" => Some("River"),
            "labwc" => Some("LabWC"),
            "openbox" => Some("Openbox"),
            "i3" => Some("i3"),
            "bspwm" => Some("bspwm"),
            "awesome" => Some("Awesome"),
            "dwm" => Some("dwm"),
            _ => None,
        };
        if detected.is_some() {
            wm = detected.map(|value| value.to_string());
            break;
        }
    }
    let mut wm = wm?;
    if let Some(session) = session_type() {
        wm.push_str(" (");
        wm.push_str(session);
        wm.push(')');
    }
    Some(wm)
}

fn session_type() -> Option<&'static str> {
    let session = env::var("XDG_SESSION_TYPE").ok()?;
    if session.eq_ignore_ascii_case("wayland") {
        Some("Wayland")
    } else if session.eq_ignore_ascii_case("x11") {
        Some("X11")
    } else {
        None
    }
}

fn shell_name() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(shell_name_inner).clone()
}

fn shell_name_inner() -> Option<String> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(shell.as_str())
        .to_string();
    let version = match name.as_str() {
        "bash" => env::var("BASH_VERSION").ok(),
        "zsh" => env::var("ZSH_VERSION").ok(),
        "fish" => env::var("FISH_VERSION").ok(),
        "nu" | "nushell" => env::var("NU_VERSION").ok(),
        _ => None,
    }
    .and_then(|value| extract_version_token(&value));
    Some(match version {
        Some(version) => format!("{name} {version}"),
        None => name,
    })
}

fn terminal_name(app: &App) -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(|| terminal_name_inner(app)).clone()
}

fn terminal_name_inner(app: &App) -> Option<String> {
    if let Ok(term) = env::var("TERM_PROGRAM") {
        let version = env::var("TERM_PROGRAM_VERSION")
            .ok()
            .and_then(|value| extract_version_token(&value));
        let name = normalize_terminal_name(&term);
        return Some(match version {
            Some(version) => format!("{name} {version}"),
            None => name,
        });
    }
    if let Ok(term) = env::var("LC_TERMINAL") {
        let version = env::var("LC_TERMINAL_VERSION")
            .ok()
            .and_then(|value| extract_version_token(&value));
        let name = normalize_terminal_name(&term);
        return Some(match version {
            Some(version) => format!("{name} {version}"),
            None => name,
        });
    }

    let mut pid = Pid::from_u32(std::process::id());
    for _ in 0..8 {
        let process = app.system.process(pid)?;
        let name = process.name().to_string_lossy();
        let name = name.as_ref();
        if let Some(display) = known_terminal_name(name) {
            let version =
                terminal_version(name, process.exe().map(|path| path.to_path_buf()).as_ref());
            return Some(match version {
                Some(version) => format!("{display} {version}"),
                None => display,
            });
        }
        pid = process.parent()?;
    }
    None
}

fn known_terminal_name(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let display = match lower.as_str() {
        "ptyxis" => "Ptyxis",
        "gnome-terminal" | "gnome-terminal-server" => "GNOME Terminal",
        "kgx" => "GNOME Console",
        "konsole" => "Konsole",
        "alacritty" => "Alacritty",
        "kitty" => "Kitty",
        "wezterm" | "wezterm-gui" => "WezTerm",
        "foot" | "footclient" => "Foot",
        "tilix" => "Tilix",
        "terminator" => "Terminator",
        "xfce4-terminal" => "XFCE Terminal",
        "mate-terminal" => "MATE Terminal",
        "lxterminal" => "LXTerminal",
        "qterminal" => "QTerminal",
        "xterm" => "XTerm",
        "st" => "st",
        "urxvt" | "rxvt" => "rxvt",
        _ => return None,
    };
    Some(display.to_string())
}

fn normalize_terminal_name(name: &str) -> String {
    if let Some(display) = known_terminal_name(name) {
        return display;
    }
    let mut chars = name.chars();
    let mut output = String::new();
    if let Some(first) = chars.next() {
        output.push(first.to_ascii_uppercase());
        output.extend(chars);
    }
    output
}

fn terminal_version(name: &str, exe: Option<&PathBuf>) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let command = match lower.as_str() {
        "gnome-terminal-server" => "gnome-terminal",
        "wezterm-gui" => "wezterm",
        _ => name,
    };
    let command = if command == name {
        exe.and_then(|path| path.to_str()).unwrap_or(command)
    } else {
        command
    };
    command_version(command, &["--version"])
}

fn package_summary() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(package_summary_inner).clone()
}

fn package_summary_inner() -> Option<String> {
    let mut parts = Vec::new();
    let timeout = Duration::from_secs(2);

    if let Some(count) = count_command_lines("rpm", &["-qa"], timeout) {
        if count > 0 {
            parts.push(format!("{count} (rpm)"));
        }
    }
    if let Some(count) =
        count_command_lines("dpkg-query", &["-f", "${binary:Package}\\n", "-W"], timeout)
    {
        if count > 0 {
            parts.push(format!("{count} (dpkg)"));
        }
    }
    if let Some(count) = count_command_lines("pacman", &["-Qq"], timeout) {
        if count > 0 {
            parts.push(format!("{count} (pacman)"));
        }
    }
    if let Some(count) = count_flatpak(timeout) {
        if count > 0 {
            parts.push(format!("{count} (flatpak)"));
        }
    }
    if let Some(count) = count_command_lines("snap", &["list"], timeout) {
        let count = count.saturating_sub(1);
        if count > 0 {
            parts.push(format!("{count} (snap)"));
        }
    }
    if let Some(count) = count_command_lines("brew", &["list", "--formula"], timeout) {
        if count > 0 {
            parts.push(format!("{count} (brew)"));
        }
    }
    if let Some(count) = count_command_lines("brew", &["list", "--cask"], timeout) {
        if count > 0 {
            parts.push(format!("{count} (brew-cask)"));
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn count_flatpak(timeout: Duration) -> Option<usize> {
    let output = run_command_with_timeout(
        "flatpak",
        &["list", "--app", "--columns=application"],
        timeout,
    )?;
    let mut count = 0;
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.eq_ignore_ascii_case("application") {
            continue;
        }
        count += 1;
    }
    Some(count)
}

fn count_command_lines(command: &str, args: &[&str], timeout: Duration) -> Option<usize> {
    if !command_exists(command) {
        return None;
    }
    let output = run_command_with_timeout(command, args, timeout)?;
    Some(
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
    )
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') {
        return Path::new(command).exists();
    }
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|path| path.join(command).exists())
}

fn command_version(command: &str, args: &[&str]) -> Option<String> {
    let output = run_command_with_timeout(command, args, Duration::from_millis(400))?;
    extract_version_token(&output)
}

fn extract_version_token(value: &str) -> Option<String> {
    for token in value.split_whitespace() {
        let mut buf = String::new();
        let mut started = false;
        for ch in token.chars() {
            if ch.is_ascii_digit() {
                started = true;
                buf.push(ch);
            } else if started && ch == '.' {
                buf.push(ch);
            } else if started {
                break;
            }
        }
        if started {
            return Some(buf.trim_end_matches('.').to_string());
        }
    }
    None
}
