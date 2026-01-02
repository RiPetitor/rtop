use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::{App, Language};

pub fn render(frame: &mut Frame, app: &App) {
    if !app.show_setup {
        return;
    }

    let area = centered_rect(72, 70, frame.area());
    frame.render_widget(Clear, area);

    let key_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(COLOR_MUTED);

    let en_style = if app.language == Language::English {
        key_style
    } else {
        hint_style
    };
    let ru_style = if app.language == Language::Russian {
        key_style
    } else {
        hint_style
    };

    let ru_label = tr(app.language, "Russian", "Русский");
    let lines = vec![
        Line::from(Span::styled(
            tr(app.language, "Setup", "Настройки"),
            label_style,
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(tr(app.language, "Language: ", "Язык: "), label_style),
            Span::styled("English", en_style),
            Span::styled("  ", hint_style),
            Span::styled(ru_label, ru_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(tr(app.language, "Theme: ", "Тема: "), label_style),
            Span::styled(tr(app.language, "(coming soon)", "(скоро)"), hint_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Layout: ", "Макет: "), label_style),
            Span::styled(tr(app.language, "(coming soon)", "(скоро)"), hint_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Columns: ", "Колонки: "), label_style),
            Span::styled(tr(app.language, "(coming soon)", "(скоро)"), hint_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Refresh rate: ", "Частота: "), label_style),
            Span::styled(tr(app.language, "(coming soon)", "(скоро)"), hint_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "GPU: ", "GPU: "), label_style),
            Span::styled(tr(app.language, "(coming soon)", "(скоро)"), hint_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Left/Right", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "toggle language", "смена языка")),
                hint_style,
            ),
            Span::styled("Esc", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "close", "закрыть")),
                hint_style,
            ),
        ]),
    ];

    let block = Block::default()
        .title(tr(app.language, "Setup", "Настройки"))
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
