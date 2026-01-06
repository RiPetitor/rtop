use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{LineGauge, Paragraph};

use super::text::tr;
use super::theme::{COLOR_MUTED, color_for_percent};
use super::{panel_block, panel_block_focused};
use crate::app::{App, Language};
use crate::utils::{fit_text, format_bytes, percent, text_width};

pub fn render_with_focus(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_cpu_panel(frame, chunks[0], app, focused);
    render_memory_panel(frame, chunks[1], app, focused);
}

fn render_cpu_panel(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let title = tr(app.language, "CPU", "CPU");
    let block = if focused {
        panel_block_focused(title)
    } else {
        panel_block(title)
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let cpu_pct = clamp_pct(app.system.global_cpu_usage());
    let total_rows = 2u16;
    let start_y = inner
        .y
        .saturating_add(inner.height.saturating_sub(total_rows) / 2);
    let gauge_area = Rect {
        x: inner.x,
        y: start_y,
        width: inner.width,
        height: 1,
    };
    render_line_gauge(
        frame,
        gauge_area,
        ratio_from_pct(cpu_pct),
        cpu_pct,
        symbols::line::THICK_HORIZONTAL,
        symbols::line::THICK_HORIZONTAL,
    );

    let metric_y = start_y.saturating_add(1);
    if metric_y < inner.y.saturating_add(inner.height) {
        let metric_area = Rect {
            x: inner.x,
            y: metric_y,
            width: inner.width,
            height: 1,
        };
        let metric_text = format!("{:>4.1}%", cpu_pct);
        render_centered_text(
            frame,
            metric_area,
            &metric_text,
            Style::default().fg(Color::White),
        );
    }
}

fn render_memory_panel(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let title = tr(app.language, "Memory", "Память");
    let block = if focused {
        panel_block_focused(title)
    } else {
        panel_block(title)
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mem_pct = clamp_pct(percent(used_mem, total_mem));
    let swap_pct = clamp_pct(percent(used_swap, total_swap));

    let (gpu_used, gpu_total) = app
        .selected_gpu()
        .and_then(|(_, gpu)| gpu.memory.as_ref())
        .map(|mem| (mem.used_bytes, mem.total_bytes))
        .unwrap_or((0, 0));
    let gpu_pct = clamp_pct(percent(gpu_used, gpu_total));

    let metrics = [
        MetricSpec {
            label: tr(app.language, "RAM", "ОЗУ"),
            used: used_mem,
            total: total_mem,
            pct: mem_pct,
        },
        MetricSpec {
            label: tr(app.language, "SWAP", "SWAP"),
            used: used_swap,
            total: total_swap,
            pct: swap_pct,
        },
        MetricSpec {
            label: tr(app.language, "GPU", "GPU"),
            used: gpu_used,
            total: gpu_total,
            pct: gpu_pct,
        },
    ];

    render_memory_metrics(frame, inner, app.language, &metrics);
}

#[derive(Clone, Copy)]
struct MetricSpec<'a> {
    label: &'a str,
    used: u64,
    total: u64,
    pct: f32,
}

fn render_memory_metrics(
    frame: &mut Frame,
    area: Rect,
    language: Language,
    metrics: &[MetricSpec<'_>],
) {
    if area.width == 0 || area.height == 0 || metrics.is_empty() {
        return;
    }

    let rows_per_metric = 2u16;
    let max_metrics = (area.height / rows_per_metric) as usize;
    if max_metrics == 0 {
        return;
    }

    let visible = &metrics[..metrics.len().min(max_metrics)];
    let max_label_width = visible
        .iter()
        .map(|metric| text_width(metric.label))
        .max()
        .unwrap_or(0) as u16;
    let label_width = max_label_width.min(area.width);
    let gauge_x = area.x.saturating_add(label_width);
    let gauge_width = area.width.saturating_sub(label_width);
    let total_rows = rows_per_metric.saturating_mul(visible.len() as u16);
    let mut y = area
        .y
        .saturating_add(area.height.saturating_sub(total_rows) / 2);
    let bottom = area.y.saturating_add(area.height);

    for metric in visible {
        let label_area = Rect {
            x: area.x,
            y,
            width: label_width.min(area.width),
            height: 1,
        };
        render_left_label(frame, label_area, metric.label);

        let gauge_area = Rect {
            x: gauge_x,
            y,
            width: gauge_width,
            height: 1,
        };
        if gauge_area.width > 1 {
            render_line_gauge(
                frame,
                gauge_area,
                ratio_u64(metric.used, metric.total),
                metric.pct,
                symbols::line::THICK_HORIZONTAL,
                symbols::line::THICK_HORIZONTAL,
            );
        }

        let metric_area = Rect {
            x: gauge_x,
            y: y.saturating_add(1),
            width: gauge_width,
            height: 1,
        };
        if metric_area.y < bottom && metric_area.width > 0 {
            let value = metric_value_text(language, metric.used, metric.total, metric.pct);
            render_centered_text(
                frame,
                metric_area,
                &value,
                Style::default().fg(Color::White),
            );
        }

        y = y.saturating_add(rows_per_metric);
        if y >= bottom {
            break;
        }
    }
}

fn render_left_label(frame: &mut Frame, area: Rect, label: &str) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let text = fit_text(label, area.width as usize);
    let paragraph = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(COLOR_MUTED),
    )));
    frame.render_widget(paragraph, area);
}

fn render_centered_text(frame: &mut Frame, area: Rect, value: &str, style: Style) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let text = fit_text(value, area.width as usize);
    let paragraph =
        Paragraph::new(Line::from(Span::styled(text, style))).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_line_gauge(
    frame: &mut Frame,
    area: Rect,
    ratio: f64,
    pct: f32,
    filled: &'static str,
    unfilled: &'static str,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let gauge = LineGauge::default()
        .ratio(ratio)
        .label(Line::from(""))
        .filled_symbol(filled)
        .unfilled_symbol(unfilled)
        .filled_style(Style::default().fg(color_for_percent(pct)))
        .unfilled_style(Style::default().fg(COLOR_MUTED));
    frame.render_widget(gauge, area);
}

fn metric_value_text(language: Language, used: u64, total: u64, pct: f32) -> String {
    if total > 0 {
        format!(
            "{}/{} {:>4.1}%",
            format_bytes(used),
            format_bytes(total),
            pct
        )
    } else {
        tr(language, "n/a", "н/д").to_string()
    }
}

fn ratio_u64(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64).clamp(0.0, 1.0)
    }
}

fn clamp_pct(pct: f32) -> f32 {
    if pct.is_finite() {
        pct.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn ratio_from_pct(pct: f32) -> f64 {
    (clamp_pct(pct) as f64 / 100.0).clamp(0.0, 1.0)
}
