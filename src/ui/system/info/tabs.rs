use std::cmp::Ordering;

use ratatui::prelude::Style;
use ratatui::text::Line;
use sysinfo::LoadAvg;

use crate::app::App;
use crate::ui::text::tr;
use crate::utils::{format_bytes, percent};

use super::layout::{push_header, push_line};

#[derive(Clone, Copy)]
pub(super) struct TabLayout {
    pub width: usize,
    pub label_width: usize,
    pub label_style: Style,
    pub value_style: Style,
    pub section_style: Style,
}

pub(super) fn push_cpu(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    layout: TabLayout,
    cpu_brand: &str,
    cpu_cores: &str,
    cpu_freq: &str,
    cpu_usage: f32,
    load: LoadAvg,
) {
    push_header(lines, tr(app.language, "CPU", "CPU"), layout.width, layout.section_style);
    push_line(
        lines,
        tr(app.language, "Model", "Модель"),
        cpu_brand.to_string(),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Cores", "Ядра"),
        cpu_cores.to_string(),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Freq", "Част."),
        cpu_freq.to_string(),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Usage", "Загр."),
        format!("{cpu_usage:.1}%"),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Load", "Нагрузка"),
        format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
}

pub(super) fn push_memory(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    layout: TabLayout,
    mem_pct: f32,
    used_mem: u64,
    total_mem: u64,
    avail_mem: u64,
    free_mem: u64,
    swap_pct: f32,
    used_swap: u64,
    total_swap: u64,
) {
    push_header(
        lines,
        tr(app.language, "Memory", "Память"),
        layout.width,
        layout.section_style,
    );
    push_line(
        lines,
        tr(app.language, "RAM", "ОЗУ"),
        format!(
            "{} / {} ({mem_pct:.0}%)",
            format_bytes(used_mem),
            format_bytes(total_mem)
        ),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Avail", "Дост."),
        format_bytes(avail_mem),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Free", "Своб."),
        format_bytes(free_mem),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
    push_line(
        lines,
        tr(app.language, "Swap", "Swap"),
        format!(
            "{} / {} ({swap_pct:.0}%)",
            format_bytes(used_swap),
            format_bytes(total_swap)
        ),
        layout.width,
        layout.label_width,
        layout.label_style,
        layout.value_style,
    );
}

pub(super) fn push_disks(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    layout: TabLayout,
    na: &str,
) {
    push_header(
        lines,
        tr(app.language, "Disks", "Диски"),
        layout.width,
        layout.section_style,
    );
    if app.disks.is_empty() {
        push_line(
            lines,
            tr(app.language, "Disk", "Диск"),
            na.to_string(),
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
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
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
        );
    }
}

pub(super) fn push_network(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    layout: TabLayout,
    net_refresh: Option<f64>,
    na: &str,
) {
    push_header(
        lines,
        tr(app.language, "Network", "Сеть"),
        layout.width,
        layout.section_style,
    );
    if app.networks.is_empty() {
        push_line(
            lines,
            tr(app.language, "Net", "Сеть"),
            na.to_string(),
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
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
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
        );
    }
}

pub(super) fn push_temps(lines: &mut Vec<Line<'static>>, app: &App, layout: TabLayout, na: &str) {
    push_header(
        lines,
        tr(app.language, "Temps", "Темп."),
        layout.width,
        layout.section_style,
    );
    if app.components.is_empty() {
        push_line(
            lines,
            tr(app.language, "Temp", "Темп."),
            na.to_string(),
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
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
                .unwrap_or(Ordering::Equal),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
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
            layout.width,
            layout.label_width,
            layout.label_style,
            layout.value_style,
        );
    }
}
