mod hardware;
mod icons;
mod os;
mod packages;
mod software;

use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use sysinfo::System;

use crate::app::{App, IconMode, SystemOverviewSnapshot, SystemTab};
use crate::ui::text::tr;
use crate::ui::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::utils::{fit_text, format_bytes, percent, text_width};

use hardware::{
    cpu_overview_line, disk_summary_lines, display_summary, gpu_summary, motherboard_summary,
    mouse_name, summarize_cpu_freq,
};
use icons::{
    ICON_BOARD, ICON_CPU, ICON_DE, ICON_DISK, ICON_DISPLAY, ICON_DISTRO, ICON_GPU, ICON_KERNEL,
    ICON_MEM, ICON_MOUSE, ICON_OS, ICON_PKG, ICON_SEP_NERD, ICON_SHELL, ICON_TERM, ICON_UPTIME,
    ICON_USER, ICON_WM, IconLabel,
};
use os::{distro_variant_line, format_uptime_long, os_release};
use packages::package_summary;
use software::{desktop_environment, shell_name, terminal_name, window_manager};

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

    let icon_mode = app.icon_mode;

    let mut lines = Vec::new();
    let push_main = |lines: &mut Vec<Line<'static>>, snapshot: &SystemOverviewSnapshot| {
        push_icon_line(
            lines,
            &ICON_USER,
            snapshot.user_host.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            &ICON_DISTRO,
            snapshot.distro_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_OS,
            snapshot.os_name.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_KERNEL,
            snapshot.kernel_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_UPTIME,
            snapshot.uptime_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            &ICON_BOARD,
            snapshot.board_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_CPU,
            snapshot.cpu_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_GPU,
            snapshot.gpu_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_MEM,
            snapshot.mem_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );

        if snapshot.disk_lines.is_empty() {
            push_icon_line(
                lines,
                &ICON_DISK,
                na.to_string(),
                width,
                icon_style,
                sep_style,
                value_style,
                icon_mode,
            );
        } else {
            for value in snapshot.disk_lines.iter().cloned() {
                push_icon_line(
                    lines,
                    &ICON_DISK,
                    value,
                    width,
                    icon_style,
                    sep_style,
                    value_style,
                    icon_mode,
                );
            }
        }

        push_icon_line(
            lines,
            &ICON_DISPLAY,
            snapshot.display_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_MOUSE,
            snapshot.mouse_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        lines.push(Line::from(""));

        push_icon_line(
            lines,
            &ICON_DE,
            snapshot.de_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_WM,
            snapshot.wm_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_SHELL,
            snapshot.shell_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_TERM,
            snapshot.terminal_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
        );
        push_icon_line(
            lines,
            &ICON_PKG,
            snapshot.package_line.clone(),
            width,
            icon_style,
            sep_style,
            value_style,
            icon_mode,
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

fn push_icon_line(
    lines: &mut Vec<Line<'static>>,
    icon: &IconLabel,
    value: String,
    width: usize,
    icon_style: Style,
    sep_style: Style,
    value_style: Style,
    icon_mode: IconMode,
) {
    let label = icon.get(icon_mode);
    let (icon_text, sep_text) = match icon_mode {
        IconMode::Nerd => (format!("{label} "), format!("{ICON_SEP_NERD} ")),
        IconMode::Text => (format!("{label} "), String::new()),
    };
    let used = text_width(&icon_text) + text_width(&sep_text);
    let max_value = width.saturating_sub(used).max(1);
    let value = fit_text(&value, max_value);
    if sep_text.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(icon_text, icon_style),
            Span::styled(value, value_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(icon_text, icon_style),
            Span::styled(sep_text, sep_style),
            Span::styled(value, value_style),
        ]));
    }
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
