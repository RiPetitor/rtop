use std::cmp::Ordering;
use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};

use super::panel_block;
use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_GOOD, COLOR_MUTED};
use crate::app::{App, GpuProcessSortKey, HighlightMode};
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

pub fn render_gpu_processes(frame: &mut Frame, area: Rect, app: &mut App) {
    app.gpu_process_order.clear();
    app.gpu_process_body = None;
    app.gpu_process_header_regions.clear();

    if area.width == 0 || area.height == 0 {
        return;
    }

    let panel_title = tr(app.language, "GPU Processes", "Процессы GPU");
    let Some(selected_id) = app.selected_gpu().map(|(_, gpu)| gpu.id.as_str()) else {
        let paragraph = Paragraph::new(tr(app.language, "No GPU selected", "GPU не выбран"))
            .block(panel_block(panel_title))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    };

    let name_map = build_name_map(app);
    let mut rows = app
        .gpu_processes
        .iter()
        .filter(|entry| entry.gpu_id == selected_id)
        .map(|entry| GpuProcessRow {
            pid: entry.pid,
            name: name_map
                .get(&entry.pid)
                .copied()
                .unwrap_or("<exited>")
                .to_string(),
            kind: entry.kind,
            sm_pct: entry.sm_pct,
            mem_pct: entry.mem_pct,
            enc_pct: entry.enc_pct,
            dec_pct: entry.dec_pct,
            fb_mb: entry.fb_mb,
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        app.gpu_process_scroll = 0;
        let paragraph = Paragraph::new(tr(app.language, "No GPU processes", "Нет процессов GPU"))
            .block(panel_block(panel_title))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    rows.sort_by(|a, b| sort_gpu_rows(a, b, app.gpu_process_sort_key, app.gpu_process_sort_dir));
    app.gpu_process_order = rows.iter().map(|row| row.pid).collect();

    let block = panel_block(panel_title);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    app.gpu_process_body = if inner.height > 1 {
        Some(Rect {
            x: inner.x,
            y: inner.y.saturating_add(1),
            width: inner.width,
            height: inner.height.saturating_sub(1),
        })
    } else {
        None
    };
    update_gpu_process_header_regions(app, area);

    let max_rows = app
        .gpu_process_body
        .map(|rect| rect.height as usize)
        .unwrap_or(0);
    app.ensure_gpu_process_visible(max_rows);
    let start = app.gpu_process_scroll.min(rows.len());
    let end = (start + max_rows).min(rows.len());
    let visible_rows = if start < end { &rows[start..end] } else { &[] };
    let table_rows = visible_rows
        .iter()
        .map(|row| {
            Row::new(vec![
                row.pid.to_string(),
                row.kind
                    .map(|kind| kind.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                format_optional_pct(row.sm_pct),
                format_optional_pct(row.mem_pct),
                format_optional_pct(row.enc_pct),
                format_optional_pct(row.dec_pct),
                format_fb_mb(row.fb_mb),
                row.name.clone(),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        gpu_header_cell(app, GpuProcessSortKey::Pid, "PID"),
        gpu_header_cell(app, GpuProcessSortKey::Kind, "Type"),
        gpu_header_cell(app, GpuProcessSortKey::Sm, "SM%"),
        gpu_header_cell(app, GpuProcessSortKey::Mem, "MEM%"),
        gpu_header_cell(app, GpuProcessSortKey::Enc, "ENC%"),
        gpu_header_cell(app, GpuProcessSortKey::Dec, "DEC%"),
        gpu_header_cell(app, GpuProcessSortKey::Vram, "VRAM"),
        gpu_header_cell(app, GpuProcessSortKey::Name, "NAME"),
    ]);

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(7),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(10),
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
    if let Some(selected_pid) = app.selected_pid
        && let Some(index) = rows.iter().position(|row| row.pid == selected_pid)
        && index >= start
        && index < end
    {
        state.select(Some(index - start));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

fn gpu_header_cell(app: &App, key: GpuProcessSortKey, label: &str) -> Cell<'static> {
    let active = app.gpu_process_sort_key == key;
    let indicator = if active {
        match app.gpu_process_sort_dir {
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

fn sort_gpu_rows(
    a: &GpuProcessRow,
    b: &GpuProcessRow,
    key: GpuProcessSortKey,
    dir: SortDir,
) -> Ordering {
    let ordering = match key {
        GpuProcessSortKey::Pid => cmp_u32(a.pid, b.pid, dir),
        GpuProcessSortKey::Kind => cmp_option_char(a.kind, b.kind, dir),
        GpuProcessSortKey::Sm => cmp_option_f32(a.sm_pct, b.sm_pct, dir),
        GpuProcessSortKey::Mem => cmp_option_f32(a.mem_pct, b.mem_pct, dir),
        GpuProcessSortKey::Enc => cmp_option_f32(a.enc_pct, b.enc_pct, dir),
        GpuProcessSortKey::Dec => cmp_option_f32(a.dec_pct, b.dec_pct, dir),
        GpuProcessSortKey::Vram => cmp_option_u64(a.fb_mb, b.fb_mb, dir),
        GpuProcessSortKey::Name => cmp_str(&a.name, &b.name, dir),
    };

    ordering.then_with(|| a.pid.cmp(&b.pid))
}

fn cmp_u32(a: u32, b: u32, dir: SortDir) -> Ordering {
    match dir {
        SortDir::Asc => a.cmp(&b),
        SortDir::Desc => b.cmp(&a),
    }
}

fn cmp_str(a: &str, b: &str, dir: SortDir) -> Ordering {
    match dir {
        SortDir::Asc => a.cmp(b),
        SortDir::Desc => b.cmp(a),
    }
}

fn cmp_option_char(a: Option<char>, b: Option<char>, dir: SortDir) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match dir {
            SortDir::Asc => a.cmp(&b),
            SortDir::Desc => b.cmp(&a),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn cmp_option_u64(a: Option<u64>, b: Option<u64>, dir: SortDir) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match dir {
            SortDir::Asc => a.cmp(&b),
            SortDir::Desc => b.cmp(&a),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn cmp_option_f32(a: Option<f32>, b: Option<f32>, dir: SortDir) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match dir {
            SortDir::Asc => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
            SortDir::Desc => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn build_name_map(app: &App) -> HashMap<u32, &str> {
    let mut map = HashMap::with_capacity(app.rows.len());
    for row in &app.rows {
        map.insert(row.pid, row.name.as_str());
    }
    map
}

fn format_optional_pct(value: Option<f32>) -> String {
    value
        .map(|pct| format!("{:>5.1}", pct))
        .unwrap_or_else(|| "  -  ".to_string())
}

fn format_fb_mb(value: Option<u64>) -> String {
    value
        .map(|mb| format_bytes(mb.saturating_mul(1024 * 1024)))
        .unwrap_or_else(|| "-".to_string())
}

struct GpuProcessRow {
    pid: u32,
    name: String,
    kind: Option<char>,
    sm_pct: Option<f32>,
    mem_pct: Option<f32>,
    enc_pct: Option<f32>,
    dec_pct: Option<f32>,
    fb_mb: Option<u64>,
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

fn update_gpu_process_header_regions(app: &mut App, area: Rect) {
    let block = panel_block("GPU Processes");
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 {
        app.gpu_process_header_regions.clear();
        return;
    }

    let spacing = 1u16;
    let constraints = [
        Constraint::Length(7),
        Constraint::Length(4),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
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
            0 => GpuProcessSortKey::Pid,
            1 => GpuProcessSortKey::Kind,
            2 => GpuProcessSortKey::Sm,
            3 => GpuProcessSortKey::Mem,
            4 => GpuProcessSortKey::Enc,
            5 => GpuProcessSortKey::Dec,
            6 => GpuProcessSortKey::Vram,
            _ => GpuProcessSortKey::Name,
        };
        regions.push(crate::app::GpuProcessHeaderRegion {
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

    app.gpu_process_header_regions = regions;
}
