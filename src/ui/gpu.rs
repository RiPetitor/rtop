use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::panel_block;
use super::processes;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::App;
use crate::data::gpu::gpu_vendor_label;
use crate::utils::{fit_text, format_bytes, percent};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    const MIN_DETAIL_HEIGHT: u16 = 8;
    const MIN_TABLE_HEIGHT: u16 = 6;

    let mut detail_height = (area.height / 2).max(MIN_DETAIL_HEIGHT);
    if area.height > MIN_TABLE_HEIGHT {
        detail_height = detail_height.min(area.height - MIN_TABLE_HEIGHT);
    } else {
        detail_height = area.height;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(detail_height), Constraint::Min(0)])
        .split(area);

    render_details(frame, chunks[0], app);
    if chunks[1].height > 0 {
        processes::render_gpu_processes(frame, chunks[1], app);
    }
}

fn render_details(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block("GPU Details");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let width = inner.width as usize;
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);
    let title_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);

    let mut lines = Vec::new();

    if !app.vram_enabled {
        lines.push(Line::from(Span::styled(
            fit_text("GPU monitoring disabled", width),
            label_style,
        )));
    } else if let Some((idx, gpu)) = app.selected_gpu() {
        let total_gpus = app.gpu_list.len();
        let name = if gpu.name.is_empty() {
            gpu_vendor_label(gpu)
        } else {
            gpu.name.clone()
        };
        let title = format!("GPU {}/{}: {}", idx + 1, total_gpus, name);
        lines.push(Line::from(Span::styled(
            fit_text(&title, width),
            title_style,
        )));

        let vendor = gpu.vendor.as_deref().unwrap_or("unknown");
        let device = gpu.device.as_deref().unwrap_or("n/a");
        push_pair(
            &mut lines,
            "Vendor",
            format!("{vendor} ({device})"),
            width,
            label_style,
            value_style,
        );
        push_pair(
            &mut lines,
            "ID    ",
            gpu.id.clone(),
            width,
            label_style,
            value_style,
        );
        push_pair(
            &mut lines,
            "Kind  ",
            format!("{:?}", gpu.kind),
            width,
            label_style,
            value_style,
        );

        let memory_line = if let Some(memory) = gpu.memory.as_ref() {
            let pct = percent(memory.used_bytes, memory.total_bytes);
            format!(
                "{} / {} ({:>4.1}%)",
                format_bytes(memory.used_bytes),
                format_bytes(memory.total_bytes),
                pct
            )
        } else {
            "n/a".to_string()
        };
        push_pair(
            &mut lines,
            "Memory",
            memory_line,
            width,
            label_style,
            value_style,
        );

        let core = format_optional_pct(gpu.telemetry.utilization_gpu_pct);
        let mem = format_optional_pct(gpu.telemetry.utilization_mem_pct);
        push_pair(
            &mut lines,
            "Util  ",
            format!("core {core} | mem {mem}"),
            width,
            label_style,
            value_style,
        );

        let temp = format_optional_temp(gpu.telemetry.temperature_c);
        let fan = format_optional_pct(gpu.telemetry.fan_speed_pct);
        push_pair(
            &mut lines,
            "Therm ",
            format!("temp {temp} | fan {fan}"),
            width,
            label_style,
            value_style,
        );

        let power = format_power(gpu.telemetry.power_draw_w, gpu.telemetry.power_limit_w);
        push_pair(&mut lines, "Power ", power, width, label_style, value_style);

        let enc = format_optional_pct(gpu.telemetry.encoder_pct);
        let dec = format_optional_pct(gpu.telemetry.decoder_pct);
        push_pair(
            &mut lines,
            "Codec ",
            format!("enc {enc} | dec {dec}"),
            width,
            label_style,
            value_style,
        );
    } else {
        lines.push(Line::from(Span::styled(
            fit_text("No GPU detected", width),
            label_style,
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn push_pair(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: String,
    width: usize,
    label_style: Style,
    value_style: Style,
) {
    let label = format!("{label}: ");
    let max_value = width.saturating_sub(label.len()).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(label, label_style),
        Span::styled(value, value_style),
    ]));
}

fn format_optional_pct(value: Option<f32>) -> String {
    value
        .map(|pct| format!("{:>4.0}%", pct))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_optional_temp(value: Option<f32>) -> String {
    value
        .map(|temp| format!("{temp:.0}C"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_power(draw: Option<f32>, limit: Option<f32>) -> String {
    match (draw, limit) {
        (Some(draw), Some(limit)) => format!("{draw:.0}W / {limit:.0}W"),
        (Some(draw), None) => format!("{draw:.0}W"),
        (None, Some(limit)) => format!("limit {limit:.0}W"),
        (None, None) => "n/a".to_string(),
    }
}
