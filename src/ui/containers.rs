use ratatui::prelude::*;
use ratatui::widgets::{Cell, HighlightSpacing, Paragraph, Row, Table, TableState};

use super::panel_block;
use super::text::tr;
use super::theme::COLOR_MUTED;
use crate::app::App;
use crate::utils::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if app.container_rows.is_empty() {
        let paragraph = Paragraph::new(tr(
            app.language,
            "No containers detected",
            "Контейнеры не обнаружены",
        ))
        .block(panel_block(tr(app.language, "Containers", "Контейнеры")))
        .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let max_rows = area.height.saturating_sub(3) as usize;
    app.ensure_container_visible(max_rows);

    let start = app.container_scroll.min(app.container_rows.len());
    let end = (start + max_rows).min(app.container_rows.len());
    let visible_rows = if start < end {
        &app.container_rows[start..end]
    } else {
        &[]
    };

    let table_rows = visible_rows
        .iter()
        .map(|row| {
            Row::new(vec![
                row.label.clone(),
                format!("{:>5.1}", row.cpu),
                format_bytes(row.mem_bytes),
                row.proc_count.to_string(),
                format_net(row.net_bytes_per_sec),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        Cell::from(tr(app.language, "CONTAINER", "КОНТЕЙНЕР")),
        Cell::from("CPU%"),
        Cell::from(tr(app.language, "MEM", "ПАМ")),
        Cell::from(tr(app.language, "PROCS", "ПРОЦ")),
        Cell::from(tr(app.language, "NET", "СЕТЬ")),
    ])
    .style(
        Style::default()
            .fg(COLOR_MUTED)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        table_rows,
        [
            Constraint::Min(14),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(panel_block(tr(app.language, "Containers", "Контейнеры")))
    .column_spacing(1)
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(40, 48, 58))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ")
    .highlight_spacing(HighlightSpacing::Always);

    let mut state = TableState::default();
    if let Some(selected) = app.container_table_state.selected()
        && selected >= start
        && selected < end
    {
        state.select(Some(selected - start));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

fn format_net(value: Option<u64>) -> String {
    let Some(bytes_per_sec) = value else {
        return "-".to_string();
    };
    const UNIT: f64 = 1024.0;
    let value = bytes_per_sec as f64;
    if value < UNIT {
        return format!("{value:.0}B/s");
    }
    let kb = value / UNIT;
    if kb < UNIT {
        return format!("{kb:.1}K/s");
    }
    let mb = kb / UNIT;
    if mb < UNIT {
        return format!("{mb:.1}M/s");
    }
    let gb = mb / UNIT;
    if gb < UNIT {
        return format!("{gb:.1}G/s");
    }
    let tb = gb / UNIT;
    format!("{tb:.1}T/s")
}
