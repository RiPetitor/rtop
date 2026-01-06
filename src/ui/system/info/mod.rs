mod hardware;
mod icons;
mod layout;
mod os;
mod overview;
mod packages;
mod software;
mod tabs;

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use sysinfo::System;

use crate::app::{App, SystemTab};
use crate::ui::text::tr;
use crate::ui::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::utils::percent;

use hardware::summarize_cpu_freq;
use overview::{OverviewLayout, ensure_snapshot, push_overview_lines};
use tabs::{TabLayout, push_cpu, push_disks, push_memory, push_network, push_temps};

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
    let label_width = width.clamp(6, 12);

    let unknown = tr(app.language, "unknown", "неизвестно");
    let na = tr(app.language, "n/a", "н/д");

    ensure_snapshot(app);

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
    let net_refresh = app.network_refresh_secs.filter(|value| *value > 0.0);

    let icon_mode = app.icon_mode;

    let mut lines = Vec::new();
    let tab_layout = TabLayout {
        width,
        label_width,
        label_style,
        value_style,
        section_style,
    };
    let overview_layout = OverviewLayout {
        width,
        icon_style,
        sep_style,
        value_style,
        icon_mode,
    };

    match app.system_tab {
        SystemTab::Overview => {
            if let Some(snapshot) = app.system_overview_snapshot.as_ref() {
                push_overview_lines(&mut lines, snapshot, overview_layout, na);
            }
        }
        SystemTab::Cpu => {
            push_cpu(
                &mut lines, app, tab_layout, &cpu_brand, &cpu_cores, &cpu_freq, cpu_usage, load,
            );
        }
        SystemTab::Memory => {
            push_memory(
                &mut lines, app, tab_layout, mem_pct, used_mem, total_mem, avail_mem, free_mem,
                swap_pct, used_swap, total_swap,
            );
        }
        SystemTab::Disks => {
            push_disks(&mut lines, app, tab_layout, na);
        }
        SystemTab::Network => {
            push_network(&mut lines, app, tab_layout, net_refresh, na);
        }
        SystemTab::Temps => {
            push_temps(&mut lines, app, tab_layout, na);
        }
    }

    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
