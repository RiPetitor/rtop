use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use sysinfo::System;

use crate::app::App;
use crate::ui::text::tr;
use crate::ui::theme::COLOR_MUTED;
use crate::utils::{fit_text, format_bytes, format_duration};

pub fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);
    let width = area.width.max(1) as usize;

    let unknown = tr(app.language, "unknown", "неизвестно");
    let host = System::host_name().unwrap_or_else(|| unknown.to_string());
    let user = app.current_user_name().unwrap_or(unknown);
    let user_host = format!("{user}@{host}");

    let os_name = System::name().unwrap_or_else(|| unknown.to_string());
    let os_version = System::os_version().unwrap_or_default();
    let os_line = if os_version.is_empty() {
        os_name
    } else {
        format!("{os_name} {os_version}")
    };

    let kernel = System::kernel_version().unwrap_or_else(|| unknown.to_string());
    let uptime = format_duration(System::uptime());
    let load = System::load_average();
    let cpu_brand = app
        .system
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_count = app.system.cpus().len();
    let arch = std::env::consts::ARCH;
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let processes = app.system.processes().len();

    let gpu_label = if let Some((_idx, gpu)) = app.selected_gpu() {
        if gpu.name.is_empty() {
            gpu.vendor.clone().unwrap_or_else(|| "GPU".to_string())
        } else {
            gpu.name.clone()
        }
    } else {
        tr(app.language, "n/a", "н/д").to_string()
    };

    let mut lines = Vec::new();
    let label_user = format!("{:<6}", tr(app.language, "User", "Польз."));
    let label_host = format!("{:<6}", tr(app.language, "Host", "Хост"));
    let label_os = format!("{:<6}", tr(app.language, "OS", "ОС"));
    let label_kernel = format!("{:<6}", tr(app.language, "Kernel", "Ядро"));
    let label_arch = format!("{:<6}", tr(app.language, "Arch", "Арх"));
    let label_uptime = format!("{:<6}", tr(app.language, "Uptime", "Аптайм"));
    let label_load = format!("{:<6}", tr(app.language, "Load", "Нагр."));
    let label_cpu = format!("{:<6}", tr(app.language, "CPU", "CPU"));
    let label_memory = format!("{:<6}", tr(app.language, "Memory", "Память"));
    let label_swap = format!("{:<6}", tr(app.language, "Swap", "Swap"));
    let label_gpu = format!("{:<6}", tr(app.language, "GPU", "GPU"));
    let label_procs = format!("{:<6}", tr(app.language, "Procs", "Проц."));

    push_line(
        &mut lines,
        &label_user,
        user_host,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_host,
        host,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_os,
        os_line,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_kernel,
        kernel,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_arch,
        arch.to_string(),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_uptime,
        uptime,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_load,
        format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_cpu,
        format!("{cpu_brand} ({cpu_count} cores)"),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_memory,
        format!("{} / {}", format_bytes(used_mem), format_bytes(total_mem)),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_swap,
        format!("{} / {}", format_bytes(used_swap), format_bytes(total_swap)),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_gpu,
        gpu_label,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_procs,
        processes.to_string(),
        width,
        label_style,
        value_style,
    );

    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn push_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: String,
    width: usize,
    label_style: Style,
    value_style: Style,
) {
    let max_value = width.saturating_sub(label.len()).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(label.to_string(), label_style),
        Span::styled(value, value_style),
    ]));
}
