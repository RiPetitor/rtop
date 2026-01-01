use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::theme::{COLOR_ACCENT, COLOR_HOT, COLOR_MUTED};
use crate::app::App;
use crate::utils::format_bytes;

pub fn render(frame: &mut Frame, app: &App) {
    let Some(confirm) = app.confirm.as_ref() else {
        return;
    };

    let area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, area);

    let title_style = Style::default().fg(COLOR_HOT).add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);

    let lines = vec![
        Line::from(Span::styled("Terminate process?", title_style)),
        Line::from(""),
        Line::from(vec![
            Span::styled("PID ", label_style),
            Span::styled(confirm.pid.to_string(), value_style),
            Span::raw("  "),
            Span::styled("CPU ", label_style),
            Span::styled(format!("{:>5.1}%", confirm.cpu), value_style),
            Span::raw("  "),
            Span::styled("MEM ", label_style),
            Span::styled(format_bytes(confirm.mem_bytes), value_style),
        ]),
        Line::from(vec![
            Span::styled("Name ", label_style),
            Span::styled(confirm.name.as_str(), value_style),
        ]),
        Line::from(vec![
            Span::styled("Status ", label_style),
            Span::styled(confirm.status.as_str(), value_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" terminate  ", label_style),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" cancel", label_style),
        ]),
    ];

    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_HOT))
        .title_style(title_style);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
