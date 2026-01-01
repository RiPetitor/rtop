use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use sysinfo::System;

use super::panel_block;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::{App, ViewMode};
use crate::utils::{format_bytes, format_duration, percent};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let load = System::load_average();
    let cpu = app.system.global_cpu_info().cpu_usage();
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let uptime = format_duration(System::uptime());
    let process_count = app.system.processes().len();
    let mem_pct = percent(used_mem, total_mem);
    let swap_pct = percent(used_swap, total_swap);

    let title_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);

    let mut first_line = vec![
        Span::styled("rtop", title_style),
        Span::raw("  "),
        Span::styled("system monitor", Style::default().fg(COLOR_MUTED)),
        Span::raw("  "),
        Span::styled("sort ", label_style),
        Span::styled(
            format!("{} {}", app.sort_key.label(), app.sort_dir.label()),
            Style::default().fg(COLOR_ACCENT),
        ),
        Span::raw("  "),
        Span::styled("view ", label_style),
        Span::styled(app.view_mode.label(), Style::default().fg(COLOR_ACCENT)),
        Span::raw("  "),
        Span::styled("highlight ", label_style),
        Span::styled(
            app.highlight_mode.label(),
            Style::default().fg(COLOR_ACCENT),
        ),
    ];
    if app.view_mode == ViewMode::Processes {
        let tree_style = if app.tree_view {
            Style::default().fg(COLOR_ACCENT)
        } else {
            Style::default().fg(COLOR_MUTED)
        };
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled("tree ", label_style));
        first_line.push(Span::styled(
            if app.tree_view { "on" } else { "off" },
            tree_style,
        ));
    }
    if let Some(filter) = app.container_filter.as_ref() {
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled("container ", label_style));
        first_line.push(Span::styled(
            filter.label(),
            Style::default().fg(COLOR_ACCENT),
        ));
    }

    let lines = vec![
        Line::from(first_line),
        Line::from(vec![
            Span::styled("CPU", label_style),
            Span::styled(format!(" {:>5.1}%  ", cpu), value_style),
            Span::styled("Load", label_style),
            Span::styled(
                format!(
                    " {:>4.2} {:>4.2} {:>4.2}  ",
                    load.one, load.five, load.fifteen
                ),
                value_style,
            ),
            Span::styled("Uptime", label_style),
            Span::styled(format!(" {}", uptime), value_style),
        ]),
        Line::from(vec![
            Span::styled("Mem", label_style),
            Span::styled(
                format!(
                    " {} / {} ({:>4.1}%)  ",
                    format_bytes(used_mem),
                    format_bytes(total_mem),
                    mem_pct
                ),
                value_style,
            ),
            Span::styled("Swap", label_style),
            Span::styled(
                format!(
                    " {} / {} ({:>4.1}%)  ",
                    format_bytes(used_swap),
                    format_bytes(total_swap),
                    swap_pct
                ),
                value_style,
            ),
            Span::styled("Procs", label_style),
            Span::styled(format!(" {}", process_count), value_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(panel_block("Summary"));
    frame.render_widget(paragraph, area);
}
