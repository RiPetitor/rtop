use ratatui::prelude::Style;
use ratatui::text::Line;
use sysinfo::System;

use crate::app::{App, IconMode, SystemOverviewSnapshot, SystemTab};
use crate::ui::text::tr;
use crate::utils::{format_bytes, percent};

use super::hardware::{
    cpu_overview_line, disk_summary_lines, display_summary, gpu_summary, motherboard_summary,
    mouse_name,
};
use super::icons::{
    ICON_BOARD, ICON_CPU, ICON_DE, ICON_DISK, ICON_DISPLAY, ICON_DISTRO, ICON_GPU, ICON_KERNEL,
    ICON_MEM, ICON_MOUSE, ICON_OS, ICON_PKG, ICON_SHELL, ICON_TERM, ICON_UPTIME, ICON_USER,
    ICON_WM,
};
use super::layout::push_icon_line;
use super::os::{distro_variant_line, format_uptime_long, os_release};
use super::packages::package_summary;
use super::software::{desktop_environment, shell_name, terminal_name, window_manager};

#[derive(Clone, Copy)]
pub(super) struct OverviewLayout {
    pub width: usize,
    pub icon_style: Style,
    pub sep_style: Style,
    pub value_style: Style,
    pub icon_mode: IconMode,
}

pub(super) fn ensure_snapshot(app: &mut App) {
    if app.system_tab == SystemTab::Overview && app.system_overview_snapshot.is_none() {
        let snapshot = build_system_overview_snapshot(app);
        app.system_overview_snapshot = Some(snapshot);
    }
}

pub(super) fn push_overview_lines(
    lines: &mut Vec<Line<'static>>,
    snapshot: &SystemOverviewSnapshot,
    layout: OverviewLayout,
    na: &str,
) {
    push_icon_line(
        lines,
        &ICON_USER,
        snapshot.user_host.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    lines.push(Line::from(""));

    push_icon_line(
        lines,
        &ICON_DISTRO,
        snapshot.distro_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_OS,
        snapshot.os_name.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_KERNEL,
        snapshot.kernel_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_UPTIME,
        snapshot.uptime_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    lines.push(Line::from(""));

    push_icon_line(
        lines,
        &ICON_BOARD,
        snapshot.board_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_CPU,
        snapshot.cpu_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_GPU,
        snapshot.gpu_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_MEM,
        snapshot.mem_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );

    if snapshot.disk_lines.is_empty() {
        push_icon_line(
            lines,
            &ICON_DISK,
            na.to_string(),
            layout.width,
            layout.icon_style,
            layout.sep_style,
            layout.value_style,
            layout.icon_mode,
        );
    } else {
        for line in &snapshot.disk_lines {
            push_icon_line(
                lines,
                &ICON_DISK,
                line.clone(),
                layout.width,
                layout.icon_style,
                layout.sep_style,
                layout.value_style,
                layout.icon_mode,
            );
        }
    }

    lines.push(Line::from(""));

    push_icon_line(
        lines,
        &ICON_DISPLAY,
        snapshot.display_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_MOUSE,
        snapshot.mouse_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_DE,
        snapshot.de_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_WM,
        snapshot.wm_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_SHELL,
        snapshot.shell_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_TERM,
        snapshot.terminal_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
    push_icon_line(
        lines,
        &ICON_PKG,
        snapshot.package_line.clone(),
        layout.width,
        layout.icon_style,
        layout.sep_style,
        layout.value_style,
        layout.icon_mode,
    );
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
