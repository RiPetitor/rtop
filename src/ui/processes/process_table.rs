use ratatui::prelude::*;
use ratatui::widgets::{Cell, Row, Table, TableState};

use super::super::panel_block;
use super::super::text::tr;
use super::super::theme::{COLOR_ACCENT, COLOR_GOOD, COLOR_MUTED};
use crate::app::{App, HighlightMode};
use crate::data::{SortDir, SortKey};
use crate::utils::{fit_text, format_bytes, format_duration_short};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let process_area = area;
    update_process_header_regions(app, process_area);
    let panel_title = if app.tree_view {
        tr(app.language, "Processes (Tree)", "Процессы (дерево)")
    } else {
        tr(app.language, "Processes", "Процессы")
    };
    let block = panel_block(panel_title);
    let inner = block.inner(process_area);
    app.process_body = if inner.width > 0 && inner.height > 1 {
        Some(Rect {
            x: inner.x,
            y: inner.y.saturating_add(1),
            width: inner.width,
            height: inner.height.saturating_sub(1),
        })
    } else {
        None
    };
    let name_width = app
        .process_header_regions
        .iter()
        .find(|region| region.key == SortKey::Name)
        .map(|region| region.rect.width as usize)
        .unwrap_or(0)
        .max(1);

    let max_rows = app
        .process_body
        .map(|rect| rect.height as usize)
        .unwrap_or(0);
    app.ensure_visible(max_rows);

    let start = app.scroll.min(app.rows.len());
    let end = (start + max_rows).min(app.rows.len());
    let visible_rows = if start < end {
        &app.rows[start..end]
    } else {
        &[]
    };

    let tree_labels = if app.tree_view {
        Some(&app.tree_labels)
    } else {
        None
    };

    let table_rows = visible_rows
        .iter()
        .map(|row| {
            let highlight = match app.highlight_mode {
                HighlightMode::CurrentUser => row.is_current_user,
                HighlightMode::NonRoot => row.is_non_root,
                HighlightMode::Gui => row.is_gui,
            };
            let name_text = tree_labels
                .and_then(|labels| labels.get(&row.pid))
                .map(|label| fit_text(label, name_width))
                .unwrap_or_else(|| row.name.clone());
            let name_cell = if highlight {
                Cell::from(name_text).style(Style::default().fg(COLOR_GOOD))
            } else {
                Cell::from(name_text)
            };
            Row::new(vec![
                Cell::from(row.pid.to_string()),
                Cell::from(row.user.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(format!("{:>5.1}", row.cpu)),
                Cell::from(format_bytes(row.mem_bytes)),
                Cell::from(format_duration_short(row.uptime_secs)),
                Cell::from(row.status.clone()),
                name_cell,
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        header_cell(app, SortKey::Pid, "PID"),
        header_cell(app, SortKey::User, "USER"),
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
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(block)
    .column_spacing(1)
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(40, 48, 58))
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    if let Some(selected) = app.table_state.selected()
        && selected >= start
        && selected < end
    {
        state.select(Some(selected - start));
    }

    frame.render_stateful_widget(table, process_area, &mut state);
}

fn header_cell(app: &App, key: SortKey, label: &str) -> Cell<'static> {
    let active = app.sort_key == key;
    let indicator = if active {
        match app.sort_dir {
            SortDir::Asc => "^",
            SortDir::Desc => "v",
        }
    } else {
        " "
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

fn update_process_header_regions(app: &mut App, area: Rect) {
    let block = panel_block("Processes");
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        app.process_header_regions.clear();
        return;
    }

    let spacing = 1u16;
    let constraints = [
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Min(10),
    ];
    let total_spacing = spacing.saturating_mul(constraints.len().saturating_sub(1) as u16);
    let layout_width = inner.width.saturating_sub(total_spacing);
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(Rect {
            x: 0,
            y: 0,
            width: layout_width,
            height: 1,
        });

    let mut regions = Vec::with_capacity(constraints.len());
    let mut x = inner.x;
    for (idx, rect) in layout.iter().enumerate() {
        let key = match idx {
            0 => SortKey::Pid,
            1 => SortKey::User,
            2 => SortKey::Cpu,
            3 => SortKey::Mem,
            4 => SortKey::Uptime,
            5 => SortKey::Status,
            _ => SortKey::Name,
        };
        regions.push(crate::app::HeaderRegion {
            key,
            rect: Rect {
                x,
                y: inner.y,
                width: rect.width,
                height: 1,
            },
        });
        x = x.saturating_add(rect.width + spacing);
    }

    app.process_header_regions = regions;
}
