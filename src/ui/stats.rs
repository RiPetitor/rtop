use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::text::tr;
use super::theme::{COLOR_MUTED, color_for_percent};
use super::{panel_block, panel_block_focused};
use crate::app::App;
use crate::utils::{format_bytes, percent, render_bar, text_width};

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

    let cpus = app.system.cpus();
    let width = inner.width.max(1) as usize;
    let mut rows = inner.height as usize;
    if rows == 0 {
        return;
    }

    let min_entry_width = 12;
    let cols = (width / min_entry_width).max(1);
    let mut max_tiles = rows * cols;
    let mut show_note = false;

    if cpus.len() > max_tiles && rows > 1 {
        show_note = true;
        rows = rows.saturating_sub(1);
        max_tiles = rows * cols;
    }

    let entry_width = (width / cols).max(1);
    let bar_width = entry_width.saturating_sub(9).max(1);
    let label_style = Style::default().fg(COLOR_MUTED);
    let value_style = Style::default().fg(Color::White);

    let mut lines = Vec::with_capacity(rows + usize::from(show_note));
    for row_idx in 0..rows {
        let mut spans = Vec::new();
        for col_idx in 0..cols {
            let idx = row_idx * cols + col_idx;
            if idx >= cpus.len() || idx >= max_tiles {
                break;
            }
            let usage = cpus[idx].cpu_usage().clamp(0.0, 100.0);
            let bar = render_bar(usage, bar_width);
            let entry_len = 3 + 1 + 4 + 1 + bar_width;
            let pad = entry_width.saturating_sub(entry_len);

            spans.push(Span::styled(format!("C{:02}", idx), label_style));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!("{:>3.0}%", usage), value_style));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                bar,
                Style::default().fg(color_for_percent(usage)),
            ));
            if pad > 0 {
                spans.push(Span::raw(" ".repeat(pad)));
            }
        }
        lines.push(Line::from(spans));
    }

    if show_note {
        let remaining = cpus.len().saturating_sub(max_tiles);
        if remaining > 0 {
            let note = match app.language {
                crate::app::Language::English => format!("+{remaining} more cores"),
                crate::app::Language::Russian => format!("+{remaining} ядер"),
            };
            let note = format!("{note:>width$}");
            lines.push(Line::from(Span::styled(note, label_style)));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
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
    let total_width = inner.width.max(1) as usize;

    let mem_pct = percent(used_mem, total_mem);
    let swap_pct = percent(used_swap, total_swap);

    // Collect GPU memory
    let (gpu_used, gpu_total) = app
        .selected_gpu()
        .and_then(|(_, gpu)| gpu.memory.as_ref())
        .map(|mem| (mem.used_bytes, mem.total_bytes))
        .unwrap_or((0, 0));
    let gpu_pct = percent(gpu_used, gpu_total);
    let has_gpu = gpu_total > 0;

    let mem_tail_raw = format!(
        "{:>5.1}% {} / {}",
        mem_pct,
        format_bytes(used_mem),
        format_bytes(total_mem)
    );
    let swap_tail_raw = format!(
        "{:>5.1}% {} / {}",
        swap_pct,
        format_bytes(used_swap),
        format_bytes(total_swap)
    );
    let gpu_tail_raw = if has_gpu {
        format!(
            "{:>5.1}% {} / {}",
            gpu_pct,
            format_bytes(gpu_used),
            format_bytes(gpu_total)
        )
    } else {
        tr(app.language, "n/a", "н/д").to_string()
    };

    let tail_len = text_width(&mem_tail_raw)
        .max(text_width(&swap_tail_raw))
        .max(text_width(&gpu_tail_raw));
    let mem_tail = format!("{mem_tail_raw:>tail_len$}");
    let swap_tail = format!("{swap_tail_raw:>tail_len$}");
    let gpu_tail = format!("{gpu_tail_raw:>tail_len$}");

    let base_len = 8 + tail_len;
    let bar_width = total_width.saturating_sub(base_len).max(1);
    let mem_bar = render_bar(mem_pct, bar_width);
    let swap_bar = render_bar(swap_pct, bar_width);
    let gpu_bar = render_bar(gpu_pct, bar_width);
    let label_style = Style::default().fg(COLOR_MUTED);

    let mem_label = format!("{:<5}", tr(app.language, "Mem", "ОЗУ"));
    let swap_label = format!("{:<5}", tr(app.language, "Swap", "Swap"));
    let gpu_label = format!("{:<5}", tr(app.language, "GPU", "GPU"));
    let mut lines = vec![
        Line::from(vec![
            Span::styled(mem_label, label_style),
            Span::styled(mem_bar, Style::default().fg(color_for_percent(mem_pct))),
            Span::styled(format!(" {mem_tail}"), label_style),
        ]),
        Line::from(vec![
            Span::styled(swap_label, label_style),
            Span::styled(swap_bar, Style::default().fg(color_for_percent(swap_pct))),
            Span::styled(format!(" {swap_tail}"), label_style),
        ]),
    ];

    if has_gpu {
        lines.push(Line::from(vec![
            Span::styled(gpu_label, label_style),
            Span::styled(gpu_bar, Style::default().fg(color_for_percent(gpu_pct))),
            Span::styled(format!(" {gpu_tail}"), label_style),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
