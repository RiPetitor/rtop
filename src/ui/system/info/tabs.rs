use std::cmp::Ordering;

use ratatui::prelude::{Color, Style};
use ratatui::text::{Line, Span};
use sysinfo::LoadAvg;

use crate::app::App;
use crate::data::cpu::CpuDetails;
use crate::data::{cpu_caches, cpu_details, lookup_cpu_codename};
use crate::ui::text::tr;
use crate::utils::{format_bytes, percent, text_width};

use super::layout::{push_header, push_line};

#[derive(Clone, Copy)]
pub(super) struct TabLayout {
    pub width: usize,
    pub label_width: usize,
    pub label_style: Style,
    pub value_style: Style,
    pub section_style: Style,
}

#[allow(clippy::too_many_arguments)]
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
    let details = cpu_details();
    let caches = cpu_caches();
    let codename = lookup_cpu_codename(&details.vendor_id, details.family, details.model);
    let na = tr(app.language, "N/A", "Н/Д");
    let is_root = CpuDetails::is_root();

    // Collect all labels to calculate max width
    let labels = [
        tr(app.language, "Name", "Имя"),
        tr(app.language, "Code Name", "Код. имя"),
        tr(app.language, "Package", "Сокет"),
        tr(app.language, "Technology", "Техпроц."),
        tr(app.language, "Specification", "Специфик."),
        tr(app.language, "Instructions", "Инструкции"),
        tr(app.language, "Core Speed", "Скор. ядра"),
        tr(app.language, "Bus Speed", "Шина"),
        tr(app.language, "Multiplier", "Множитель"),
        tr(app.language, "Cores", "Ядра"),
        tr(app.language, "Usage", "Загр."),
        tr(app.language, "Load", "Нагрузка"),
        "L1 Data",
        "L2",
        "L3",
    ];

    // Calculate max label width
    let max_label_width = labels.iter().map(|s| text_width(s)).max().unwrap_or(12) + 2; // Add padding

    let label_width = max_label_width.min(layout.width / 3);

    // Section: Processor
    push_header(
        lines,
        tr(app.language, "Processor", "Процессор"),
        layout.width,
        layout.section_style,
    );

    // Name
    push_line(
        lines,
        tr(app.language, "Name", "Имя"),
        cpu_brand.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Code Name
    let codename_str = codename.as_ref().map(|c| c.codename).unwrap_or(na);
    push_line(
        lines,
        tr(app.language, "Code Name", "Код. имя"),
        codename_str.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Package
    let package_str = codename.as_ref().map(|c| c.package).unwrap_or(na);
    push_line(
        lines,
        tr(app.language, "Package", "Сокет"),
        package_str.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Technology
    let tech_str = codename.as_ref().map(|c| c.technology).unwrap_or(na);
    push_line(
        lines,
        tr(app.language, "Technology", "Техпроц."),
        tech_str.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Family/Model/Stepping
    push_line(
        lines,
        tr(app.language, "Specification", "Специфик."),
        details.family_model_stepping(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Instructions
    let instructions = details.key_instructions();
    if !instructions.is_empty() {
        push_line(
            lines,
            tr(app.language, "Instructions", "Инструкции"),
            instructions.join(", "),
            layout.width,
            label_width,
            layout.label_style,
            layout.value_style,
        );
    }

    // Section: Clocks
    push_header(
        lines,
        tr(app.language, "Clocks", "Частоты"),
        layout.width,
        layout.section_style,
    );

    // Core Speed
    push_line(
        lines,
        tr(app.language, "Core Speed", "Скор. ядра"),
        cpu_freq.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Bus Speed (100 MHz for modern CPUs)
    push_line(
        lines,
        tr(app.language, "Bus Speed", "Шина"),
        "100.00 MHz".to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Multiplier
    let multiplier_value = if is_root {
        // TODO: Read actual multiplier from MSR when running as root
        tr(app.language, "N/A (MSR)", "Н/Д (MSR)").to_string()
    } else {
        tr(app.language, "need root", "нужен root").to_string()
    };
    push_line(
        lines,
        tr(app.language, "Multiplier", "Множитель"),
        multiplier_value,
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Section: Cores
    push_header(
        lines,
        tr(app.language, "Cores", "Ядра"),
        layout.width,
        layout.section_style,
    );

    push_line(
        lines,
        tr(app.language, "Cores", "Ядра"),
        cpu_cores.to_string(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    push_line(
        lines,
        tr(app.language, "Usage", "Загр."),
        format!("{cpu_usage:.1}%"),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    push_line(
        lines,
        tr(app.language, "Load", "Нагрузка"),
        format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Section: Cache
    push_header(
        lines,
        tr(app.language, "Cache", "Кэш"),
        layout.width,
        layout.section_style,
    );

    push_line(
        lines,
        "L1 Data",
        caches.format_l1(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    push_line(
        lines,
        "L2",
        caches.format_l2(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    push_line(
        lines,
        "L3",
        caches.format_l3(),
        layout.width,
        label_width,
        layout.label_style,
        layout.value_style,
    );

    // Root access hint
    if !is_root {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            tr(
                app.language,
                "Run as root for more details (voltage, multiplier)",
                "Запустите от root для деталей (напряжение, множитель)",
            ),
            Style::default().fg(Color::Yellow),
        )));
    }
}

#[allow(clippy::too_many_arguments)]
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

pub(super) fn push_disks(lines: &mut Vec<Line<'static>>, app: &App, layout: TabLayout, na: &str) {
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
            (Some(a_temp), Some(b_temp)) => b_temp.partial_cmp(a_temp).unwrap_or(Ordering::Equal),
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
