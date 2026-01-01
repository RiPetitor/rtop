use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};

use super::panel_block;
use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED, color_for_percent};
use crate::app::App;
use crate::data::gpu::gpu_vendor_label;
use crate::utils::{fit_text, format_bytes, percent, render_bar, text_width};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_cpu_panel(frame, chunks[0], app);
    render_memory_panel(frame, chunks[1], app);
    render_gpu_panel(frame, chunks[2], app);
}

fn render_cpu_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block("CPU");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let cpus = app.system.cpus();
    let cols = (inner.width / 10).max(1);
    let mut gauge_height = inner.height;
    let mut rows = (gauge_height / 3).max(1);
    let mut max_tiles = (cols * rows) as usize;
    let mut show_note = false;

    if cpus.len() > max_tiles && gauge_height > 1 {
        show_note = true;
        gauge_height = gauge_height.saturating_sub(1);
        rows = (gauge_height / 3).max(1);
        max_tiles = (cols * rows) as usize;
    }

    if gauge_height == 0 {
        return;
    }

    let tile_width = inner.width / cols;
    let tile_height = gauge_height / rows;

    for (idx, cpu) in cpus.iter().enumerate().take(max_tiles) {
        let col = (idx as u16) % cols;
        let row = (idx as u16) / cols;
        let x = inner.x + col * tile_width;
        let y = inner.y + row * tile_height;
        let rect = Rect {
            x,
            y,
            width: tile_width,
            height: tile_height,
        };

        let usage = cpu.cpu_usage().clamp(0.0, 100.0);
        let label = Span::styled(format!("C{:02}", idx), Style::default().fg(COLOR_MUTED));
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .title(label),
            )
            .gauge_style(Style::default().fg(color_for_percent(usage)))
            .label(Span::styled(
                format!("{:>3.0}%", usage),
                Style::default().fg(Color::White),
            ))
            .ratio(usage as f64 / 100.0);

        frame.render_widget(gauge, rect);
    }

    if show_note {
        let remaining = cpus.len().saturating_sub(max_tiles);
        let note_area = Rect {
            x: inner.x,
            y: inner.y + gauge_height,
            width: inner.width,
            height: 1,
        };
        if remaining > 0 {
            let note = Paragraph::new(format!("+{remaining} more cores"))
                .style(Style::default().fg(COLOR_MUTED))
                .alignment(Alignment::Right);
            frame.render_widget(note, note_area);
        }
    }
}

fn render_memory_panel(frame: &mut Frame, area: Rect, app: &App) {
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let block = panel_block("Memory");
    let inner = block.inner(area);
    let total_width = inner.width.max(1) as usize;

    let mem_pct = percent(used_mem, total_mem);
    let swap_pct = percent(used_swap, total_swap);

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
    let tail_len = text_width(&mem_tail_raw).max(text_width(&swap_tail_raw));
    let mem_tail = format!("{mem_tail_raw:>tail_len$}");
    let swap_tail = format!("{swap_tail_raw:>tail_len$}");

    let base_len = 8 + tail_len;
    let bar_width = total_width.saturating_sub(base_len).max(1);
    let mem_bar = render_bar(mem_pct, bar_width);
    let swap_bar = render_bar(swap_pct, bar_width);
    let label_style = Style::default().fg(COLOR_MUTED);

    let lines = vec![
        Line::from(vec![
            Span::styled("Mem  [", label_style),
            Span::styled(mem_bar, Style::default().fg(color_for_percent(mem_pct))),
            Span::styled(format!("] {mem_tail}"), label_style),
        ]),
        Line::from(vec![
            Span::styled("Swap [", label_style),
            Span::styled(swap_bar, Style::default().fg(color_for_percent(swap_pct))),
            Span::styled(format!("] {swap_tail}"), label_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_gpu_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block("GPU");
    let inner = block.inner(area);
    let total_width = inner.width.max(1) as usize;
    let label_style = Style::default().fg(COLOR_MUTED);
    let title_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);

    let mut lines = Vec::new();

    if !app.vram_enabled {
        lines.push(Line::from(Span::styled(
            fit_text("GPU: off", total_width),
            label_style,
        )));
        lines.push(Line::from(Span::styled(
            fit_text("vRAM: off", total_width),
            label_style,
        )));
    } else if let Some((_idx, gpu)) = app.selected_gpu() {
        let label = format!("GPU: {}", gpu_vendor_label(gpu));
        lines.push(Line::from(Span::styled(
            fit_text(&label, total_width),
            title_style,
        )));

        let (pct, tail_raw, bar_style) = if let Some(memory) = gpu.memory.as_ref() {
            let pct = percent(memory.used_bytes, memory.total_bytes);
            (
                pct,
                format!(
                    "{:>5.1}% {} / {}",
                    pct,
                    format_bytes(memory.used_bytes),
                    format_bytes(memory.total_bytes)
                ),
                Style::default().fg(color_for_percent(pct)),
            )
        } else {
            (0.0, "n/a".to_string(), Style::default().fg(COLOR_MUTED))
        };

        let tail_len = text_width(&tail_raw);
        let base_len = 8 + tail_len;
        let bar_width = total_width.saturating_sub(base_len).max(1);
        let vram_bar = render_bar(pct, bar_width);
        let vram_tail = format!("{tail_raw:>tail_len$}");
        lines.push(Line::from(vec![
            Span::styled("vRAM [", label_style),
            Span::styled(vram_bar, bar_style),
            Span::styled(format!("] {vram_tail}"), label_style),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            fit_text("GPU: not found", total_width),
            label_style,
        )));
    }

    let max_lines = inner.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
