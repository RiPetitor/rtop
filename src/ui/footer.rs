use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::{App, ViewMode};

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
        let enter_label = if app.view_mode == ViewMode::Container {
            tr(app.language, "drill", "внутрь")
        } else {
            tr(app.language, "terminate", "завершить")
        };
        let mut second_line = vec![
            Span::styled("up/down", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "move", "перемест.")),
                hint_style,
            ),
            Span::styled("left/right", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "column", "колонка")),
                hint_style,
            ),
            Span::styled("space", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "order", "порядок")),
                hint_style,
            ),
            Span::styled("enter", key_style),
            Span::styled(format!(" {enter_label}  "), hint_style),
            Span::styled("t", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "tree", "дерево")),
                hint_style,
            ),
            Span::styled("tab", key_style),
            Span::styled(format!(" {}", tr(app.language, "view", "вид")), hint_style),
        ];
        if app.container_filter.is_some() {
            second_line.push(Span::styled("  ", hint_style));
            second_line.push(Span::styled("b/esc", key_style));
            second_line.push(Span::styled(
                format!(" {}", tr(app.language, "back", "назад")),
                hint_style,
            ));
        }
        vec![
            Line::from(vec![
                Span::styled("q", key_style),
                Span::styled(
                    format!(" {}  ", tr(app.language, "quit", "выход")),
                    hint_style,
                ),
                Span::styled("r", key_style),
                Span::styled(
                    format!(" {}  ", tr(app.language, "refresh", "обновить")),
                    hint_style,
                ),
                Span::styled("F2", key_style),
                Span::styled(
                    format!(" {}  ", tr(app.language, "setup", "настройки")),
                    hint_style,
                ),
                Span::styled("F12", key_style),
                Span::styled(
                    format!(" {}", tr(app.language, "help", "справка")),
                    hint_style,
                ),
            ]),
            Line::from(second_line),
        ]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(COLOR_BORDER));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
