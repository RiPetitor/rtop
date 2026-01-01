use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    if !app.show_help {
        return;
    }

    let area = centered_rect(72, 70, frame.size());
    frame.render_widget(Clear, area);

    let key_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(COLOR_MUTED);

    let lines = vec![
        Line::from(Span::styled("Quick Keys", label_style)),
        Line::from(vec![
            Span::styled("F2", key_style),
            Span::styled(" setup  ", hint_style),
            Span::styled("F12", key_style),
            Span::styled(" help  ", hint_style),
            Span::styled("q", key_style),
            Span::styled(" quit", hint_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", label_style)),
        Line::from(vec![
            Span::styled("Up/Down", key_style),
            Span::styled(" move  ", hint_style),
            Span::styled("Enter", key_style),
            Span::styled(" action  ", hint_style),
            Span::styled("Esc", key_style),
            Span::styled(" back/close", hint_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Sorting", label_style)),
        Line::from(vec![
            Span::styled("Left/Right", key_style),
            Span::styled(" column  ", hint_style),
            Span::styled("Space", key_style),
            Span::styled(" order  ", hint_style),
            Span::styled("Mouse", key_style),
            Span::styled(" header sort", hint_style),
        ]),
        Line::from(vec![
            Span::styled("c/m/p/n/u", key_style),
            Span::styled(" quick sort  ", hint_style),
            Span::styled("h", key_style),
            Span::styled(" highlight", hint_style),
        ]),
        Line::from(vec![
            Span::styled("Tree mode", label_style),
            Span::styled(" PID order only", hint_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Views", label_style)),
        Line::from(vec![
            Span::styled("1", key_style),
            Span::styled(" overview  ", hint_style),
            Span::styled("2", key_style),
            Span::styled(" system  ", hint_style),
            Span::styled("3", key_style),
            Span::styled(" gpu  ", hint_style),
            Span::styled("4", key_style),
            Span::styled(" containers  ", hint_style),
            Span::styled("Tab", key_style),
            Span::styled(" cycle", hint_style),
        ]),
        Line::from(vec![
            Span::styled("t", key_style),
            Span::styled(" tree (Processes)", hint_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("GPU", label_style)),
        Line::from(vec![
            Span::styled("g/G", key_style),
            Span::styled(" select GPU", hint_style),
        ]),
    ];

    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .title_style(
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

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
