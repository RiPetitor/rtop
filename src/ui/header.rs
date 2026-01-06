use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use sysinfo::System;

use super::panel_block;
use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::{App, HighlightMode, ViewMode};
use crate::utils::{format_bytes, format_duration, percent};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let cpu = app.system.global_cpu_usage();
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

    let view_label = match app.view_mode {
        ViewMode::Overview => tr(app.language, "Overview", "Обзор"),
        ViewMode::Processes => tr(app.language, "Processes", "Процессы"),
        ViewMode::GpuFocus => tr(app.language, "GPU", "GPU"),
        ViewMode::SystemInfo => tr(app.language, "System", "Система"),
        ViewMode::Container => tr(app.language, "Containers", "Контейнеры"),
    };
    let highlight_label = match app.highlight_mode {
        HighlightMode::CurrentUser => tr(app.language, "user", "польз."),
        HighlightMode::NonRoot => tr(app.language, "non-root", "не-root"),
        HighlightMode::Gui => tr(app.language, "gui", "gui"),
    };

    let mut first_line = vec![
        Span::styled("rtop", title_style),
        Span::raw("  "),
        Span::styled(
            tr(app.language, "system monitor", "монитор системы"),
            Style::default().fg(COLOR_MUTED),
        ),
        Span::raw("  "),
        Span::styled(tr(app.language, "sort ", "сорт "), label_style),
        Span::styled(
            format!("{} {}", app.sort_key.label(), app.sort_dir.label()),
            Style::default().fg(COLOR_ACCENT),
        ),
        Span::raw("  "),
        Span::styled(tr(app.language, "view ", "вид "), label_style),
        Span::styled(view_label, Style::default().fg(COLOR_ACCENT)),
        Span::raw("  "),
        Span::styled(tr(app.language, "highlight ", "подсветка "), label_style),
        Span::styled(highlight_label, Style::default().fg(COLOR_ACCENT)),
    ];
    if app.view_mode == ViewMode::Processes {
        let tree_style = if app.tree_view {
            Style::default().fg(COLOR_ACCENT)
        } else {
            Style::default().fg(COLOR_MUTED)
        };
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled(
            tr(app.language, "tree ", "дерево "),
            label_style,
        ));
        first_line.push(Span::styled(
            if app.tree_view {
                tr(app.language, "on", "вкл")
            } else {
                tr(app.language, "off", "выкл")
            },
            tree_style,
        ));
    }
    if let Some(filter) = app.container_filter.as_ref() {
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled(
            tr(app.language, "container ", "контейнер "),
            label_style,
        ));
        first_line.push(Span::styled(
            filter.label(),
            Style::default().fg(COLOR_ACCENT),
        ));
    }
    if !app.process_filter.is_empty() {
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled(
            tr(app.language, "filter ", "фильтр "),
            label_style,
        ));
        first_line.push(Span::styled(
            app.process_filter.as_str(),
            Style::default().fg(COLOR_ACCENT),
        ));
    }

    let lines = vec![
        Line::from(first_line),
        Line::from(vec![
            Span::styled(tr(app.language, "CPU", "CPU"), label_style),
            Span::styled(format!(" {:>5.1}%  ", cpu), value_style),
            Span::styled(tr(app.language, "Uptime", "Аптайм"), label_style),
            Span::styled(format!(" {}", uptime), value_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Mem", "ОЗУ"), label_style),
            Span::styled(
                format!(
                    " {} / {} ({:>4.1}%)  ",
                    format_bytes(used_mem),
                    format_bytes(total_mem),
                    mem_pct
                ),
                value_style,
            ),
            Span::styled(tr(app.language, "Swap", "Swap"), label_style),
            Span::styled(
                format!(
                    " {} / {} ({:>4.1}%)  ",
                    format_bytes(used_swap),
                    format_bytes(total_swap),
                    swap_pct
                ),
                value_style,
            ),
            Span::styled(tr(app.language, "Procs", "Проц."), label_style),
            Span::styled(format!(" {}", process_count), value_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(panel_block("Summary"));
    frame.render_widget(paragraph, area);
}
