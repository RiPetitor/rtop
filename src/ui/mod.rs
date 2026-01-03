mod confirm;
mod containers;
mod footer;
mod gpu;
mod header;
mod help;
mod processes;
mod setup;
mod stats;
mod system;
mod text;
pub mod theme;
mod widgets;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, ViewMode};
use text::tr;
use theme::COLOR_BORDER;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.process_header_regions.clear();
    app.process_body = None;
    app.gpu_process_header_regions.clear();
    app.gpu_process_body = None;
    app.gpu_process_order.clear();
    let size = frame.area();
    if size.width < 60 || size.height < 22 {
        let msg = Paragraph::new(tr(
            app.language,
            "Terminal too small. Resize to at least 60x22.",
            "Терминал слишком мал. Увеличьте до 60x22 минимум.",
        ))
        .block(panel_block("rtop"))
        .alignment(Alignment::Center);
        frame.render_widget(msg, size);
        return;
    }

    match app.view_mode {
        ViewMode::Overview => render_overview(frame, app, size),
        ViewMode::Processes => render_processes_only(frame, app, size),
        ViewMode::GpuFocus => render_gpu_focus(frame, app, size),
        ViewMode::SystemInfo => render_system_info(frame, app, size),
        ViewMode::Container => render_containers(frame, app, size),
    }
}

pub fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(COLOR_BORDER))
        .title_style(
            Style::default()
                .fg(theme::COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        )
}

pub fn panel_block_focused(title: &str) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::COLOR_ACCENT))
        .title_style(
            Style::default()
                .fg(theme::COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        )
}

fn render_overview(frame: &mut Frame, app: &mut App, size: Rect) {
    let header_height = 5;
    let footer_height = 4;

    // Если Processes развёрнут - показать только его
    if app.processes_expanded {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(8),
                Constraint::Length(footer_height),
            ])
            .split(size);

        header::render(frame, chunks[0], app);
        processes::render_with_focus(frame, chunks[1], app, true);
        footer::render(frame, chunks[2], app);
        confirm::render(frame, app);
        help::render(frame, app);
        setup::render(frame, app);
        return;
    }

    // Обычный режим
    let min_process_height = 8;
    let available = size
        .height
        .saturating_sub(header_height + footer_height + min_process_height);
    let cpu_height = available.clamp(5, 9);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(cpu_height),
            Constraint::Min(8),
            Constraint::Length(footer_height),
        ])
        .split(size);

    header::render(frame, chunks[0], app);
    stats::render_with_focus(frame, chunks[1], app, false);
    processes::render_with_focus(frame, chunks[2], app, app.processes_focused);
    footer::render(frame, chunks[3], app);
    confirm::render(frame, app);
    help::render(frame, app);
    setup::render(frame, app);
}

fn render_processes_only(frame: &mut Frame, app: &mut App, size: Rect) {
    let header_height = 5;
    let footer_height = 4;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(8),
            Constraint::Length(footer_height),
        ])
        .split(size);

    header::render(frame, chunks[0], app);
    processes::render(frame, chunks[1], app);
    footer::render(frame, chunks[2], app);
    confirm::render(frame, app);
    help::render(frame, app);
    setup::render(frame, app);
}

fn render_gpu_focus(frame: &mut Frame, app: &mut App, size: Rect) {
    let header_height = 5;
    let footer_height = 4;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(8),
            Constraint::Length(footer_height),
        ])
        .split(size);

    header::render(frame, chunks[0], app);
    gpu::render(frame, chunks[1], app);
    footer::render(frame, chunks[2], app);
    confirm::render(frame, app);
    help::render(frame, app);
    setup::render(frame, app);
}

fn render_system_info(frame: &mut Frame, app: &mut App, size: Rect) {
    let header_height = 5;
    let footer_height = 4;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(8),
            Constraint::Length(footer_height),
        ])
        .split(size);

    header::render(frame, chunks[0], app);
    system::render(frame, chunks[1], app);
    footer::render(frame, chunks[2], app);
    confirm::render(frame, app);
    help::render(frame, app);
    setup::render(frame, app);
}

fn render_containers(frame: &mut Frame, app: &mut App, size: Rect) {
    let header_height = 5;
    let footer_height = 4;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(8),
            Constraint::Length(footer_height),
        ])
        .split(size);

    header::render(frame, chunks[0], app);
    containers::render(frame, chunks[1], app);
    footer::render(frame, chunks[2], app);
    confirm::render(frame, app);
    help::render(frame, app);
    setup::render(frame, app);
}
