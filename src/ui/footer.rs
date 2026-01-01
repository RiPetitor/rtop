use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let key_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(COLOR_MUTED);

    let lines = if let Some(status) = app.status.as_ref() {
        vec![Line::from(Span::styled(
            status.text.as_str(),
            status.level.style(),
        ))]
    } else {
        vec![
            Line::from(vec![
                Span::styled("q", key_style),
                Span::styled(" quit  ", hint_style),
                Span::styled("r", key_style),
                Span::styled(" refresh  ", hint_style),
                Span::styled("up/down", key_style),
                Span::styled(" move  ", hint_style),
                Span::styled("left/right", key_style),
                Span::styled(" column  ", hint_style),
                Span::styled("space", key_style),
                Span::styled(" order", hint_style),
            ]),
            Line::from(vec![
                Span::styled("enter", key_style),
                Span::styled(" terminate  ", hint_style),
                Span::styled("c/m/p/n", key_style),
                Span::styled(" quick sort  ", hint_style),
                Span::styled("g/G", key_style),
                Span::styled(" gpu", hint_style),
            ]),
        ]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(COLOR_BORDER));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
