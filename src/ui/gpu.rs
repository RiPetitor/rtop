use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::panel_block;
use super::processes;
use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_MUTED, color_for_percent};
use crate::app::App;
use crate::data::gpu::{gpu_vendor_label, short_device_name};
use crate::utils::{fit_text, format_bytes, percent, render_bar};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    const MIN_DETAIL_HEIGHT: u16 = 7;
    const MIN_TABLE_HEIGHT: u16 = 6;

    let detail_height = if area.height > MIN_DETAIL_HEIGHT + MIN_TABLE_HEIGHT {
        MIN_DETAIL_HEIGHT
    } else {
        area.height
            .saturating_sub(MIN_TABLE_HEIGHT)
            .max(MIN_DETAIL_HEIGHT)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(detail_height), Constraint::Min(0)])
        .split(area);

    render_dashboard(frame, chunks[0], app);
    if chunks[1].height > 0 {
        processes::render_gpu_processes(frame, chunks[1], app);
    }
}

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block(tr(app.language, "GPU Dashboard", "Панель GPU"));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let width = inner.width as usize;
    let label_style = Style::default().fg(COLOR_MUTED);
    let value_style = Style::default().fg(Color::White);
    let title_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);

    let mut lines = Vec::new();

    if !app.vram_enabled {
        lines.push(Line::from(Span::styled(
            fit_text(
                tr(
                    app.language,
                    "GPU monitoring disabled",
                    "Мониторинг GPU отключен",
                ),
                width,
            ),
            label_style,
        )));
    } else if let Some((idx, gpu)) = app.selected_gpu() {
        let total_gpus = app.gpu_list.len();
        let na_label = tr(app.language, "n/a", "н/д");

        // Краткое имя: "AMD RX 7700 XT" или "[1/2] AMD RX 7700 XT"
        let vendor_short = gpu_vendor_label(gpu);
        let device_name = gpu.device.as_deref().unwrap_or(&gpu.name);
        let device_short = short_device_name(device_name);
        let gpu_label = if total_gpus > 1 {
            format!(
                "[{}/{}] {} {}",
                idx + 1,
                total_gpus,
                vendor_short,
                device_short
            )
        } else {
            format!("{} {}", vendor_short, device_short)
        };

        // Выравнивание: все метки одинаковой ширины
        let label_width = gpu_label.len() + 2;
        let bar_width = calc_bar_width(width, 35);

        // Строка 1: GPU название + бар утилизации | температура | мощность
        let util_pct = gpu.telemetry.utilization_gpu_pct.unwrap_or(0.0);
        let util_bar = render_bar(util_pct, bar_width);
        let temp_str = gpu
            .telemetry
            .temperature_c
            .map(|t| format!("{:.0}°C", t))
            .unwrap_or_else(|| na_label.to_string());
        let power_str = format_power(
            gpu.telemetry.power_draw_w,
            gpu.telemetry.power_limit_w,
            na_label,
        );

        lines.push(Line::from(vec![
            Span::styled(format!("{:<label_width$}", gpu_label), title_style),
            Span::styled(util_bar, Style::default().fg(color_for_percent(util_pct))),
            Span::styled(format!(" {:>3.0}%", util_pct), value_style),
            Span::styled(" | ", label_style),
            Span::styled(temp_str, value_style),
            Span::styled(" | ", label_style),
            Span::styled(power_str, value_style),
        ]));

        // Строка 2: VRAM
        if let Some(memory) = gpu.memory.as_ref() {
            let mem_pct = percent(memory.used_bytes, memory.total_bytes);
            let mem_bar = render_bar(mem_pct, bar_width);
            let mem_info = format!(
                "{} / {}",
                format_bytes(memory.used_bytes),
                format_bytes(memory.total_bytes)
            );
            lines.push(Line::from(vec![
                Span::styled(format!("{:<label_width$}", "VRAM"), label_style),
                Span::styled(mem_bar, Style::default().fg(color_for_percent(mem_pct))),
                Span::styled(format!(" {}", mem_info), value_style),
            ]));
        }

        // Строка 3: Encoder + бар | Decoder только процент
        let enc_pct = gpu.telemetry.encoder_pct.unwrap_or(0.0);
        let dec_pct = gpu.telemetry.decoder_pct.unwrap_or(0.0);
        let enc_bar = render_bar(enc_pct, bar_width);

        lines.push(Line::from(vec![
            Span::styled(format!("{:<label_width$}", "Encoder"), label_style),
            Span::styled(enc_bar, Style::default().fg(color_for_percent(enc_pct))),
            Span::styled(format!(" {:>3.0}%", enc_pct), value_style),
            Span::styled(" | Decoder ", label_style),
            Span::styled(format!("{:>3.0}%", dec_pct), value_style),
        ]));

        // Строка 4: Fan
        if let Some(fan_pct) = gpu.telemetry.fan_speed_pct {
            let fan_bar = render_bar(fan_pct, bar_width);
            lines.push(Line::from(vec![
                Span::styled(format!("{:<label_width$}", "Fan"), label_style),
                Span::styled(fan_bar, Style::default().fg(color_for_percent(fan_pct))),
                Span::styled(format!(" {:>3.0}%", fan_pct), value_style),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            fit_text(
                tr(app.language, "No GPU detected", "GPU не обнаружен"),
                width,
            ),
            label_style,
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn calc_bar_width(total_width: usize, min_tail: usize) -> usize {
    total_width.saturating_sub(min_tail).clamp(10, 24)
}

fn format_power(draw: Option<f32>, limit: Option<f32>, na_label: &str) -> String {
    match (draw, limit) {
        (Some(draw), Some(limit)) => format!("{:.0}W/{:.0}W", draw, limit),
        (Some(draw), None) => format!("{:.0}W", draw),
        (None, Some(limit)) => format!("/{:.0}W", limit),
        (None, None) => na_label.to_string(),
    }
}
