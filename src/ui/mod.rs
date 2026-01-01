mod confirm;
mod footer;
mod header;
mod processes;
mod stats;
pub mod theme;
mod widgets;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use theme::COLOR_BORDER;

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.size();
    if size.width < 60 || size.height < 22 {
        let msg = Paragraph::new("Terminal too small. Resize to at least 60x22.")
            .block(panel_block("rtop"))
            .alignment(Alignment::Center);
        frame.render_widget(msg, size);
        return;
    }

    let header_height = 5;
    let footer_height = 4;
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
    stats::render(frame, chunks[1], app);
    processes::render(frame, chunks[2], app);
    footer::render(frame, chunks[3], app);
    confirm::render(frame, app);
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
