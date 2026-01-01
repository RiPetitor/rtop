use ratatui::prelude::*;
use ratatui::widgets::{Cell, HighlightSpacing, Row, Table, TableState};

use super::panel_block;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::App;
use crate::data::SortKey;
use crate::utils::{format_bytes, format_duration_short};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let max_rows = area.height.saturating_sub(3) as usize;
    app.ensure_visible(max_rows);

    let start = app.scroll.min(app.rows.len());
    let end = (start + max_rows).min(app.rows.len());
    let visible_rows = if start < end {
        &app.rows[start..end]
    } else {
        &[]
    };

    let table_rows = visible_rows
        .iter()
        .map(|row| {
            Row::new(vec![
                row.pid.to_string(),
                format!("{:>5.1}", row.cpu),
                format_bytes(row.mem_bytes),
                format_duration_short(row.uptime_secs),
                row.status.clone(),
                row.name.clone(),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        header_cell(app, SortKey::Pid, "PID"),
        header_cell(app, SortKey::Cpu, "CPU%"),
        header_cell(app, SortKey::Mem, "MEM"),
        header_cell(app, SortKey::Uptime, "UPTIME"),
        header_cell(app, SortKey::Status, "STAT"),
        header_cell(app, SortKey::Name, "NAME"),
    ]);

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(panel_block("Processes"))
    .column_spacing(1)
    .highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(40, 48, 58))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ")
    .highlight_spacing(HighlightSpacing::Always);

    let mut state = TableState::default();
    if let Some(selected) = app.table_state.selected() {
        if selected >= start && selected < end {
            state.select(Some(selected - start));
        }
    }

    frame.render_stateful_widget(table, area, &mut state);
}

fn header_cell(app: &App, key: SortKey, label: &str) -> Cell<'static> {
    let active = app.sort_key == key;
    let indicator = if active {
        match app.sort_dir {
            crate::data::SortDir::Asc => " ^",
            crate::data::SortDir::Desc => " v",
        }
    } else {
        "  "
    };

    let style = if active {
        Style::default()
            .fg(COLOR_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(COLOR_MUTED)
            .add_modifier(Modifier::BOLD)
    };

    Cell::from(format!("{label}{indicator}")).style(style)
}
