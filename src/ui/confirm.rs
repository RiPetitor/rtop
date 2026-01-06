use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_HOT, COLOR_MUTED};
use super::widgets::centered_rect;
use crate::app::App;
use crate::utils::format_bytes;

pub fn render(frame: &mut Frame, app: &App) {
    let Some(confirm) = app.confirm.as_ref() else {
        return;
    };

    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    let title_style = Style::default().fg(COLOR_HOT).add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);

    let lines = vec![
        Line::from(Span::styled(
            tr(app.language, "Terminate process?", "Завершить процесс?"),
            title_style,
        )),
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
            Span::styled(tr(app.language, "Name ", "Имя "), label_style),
            Span::styled(confirm.name.as_str(), value_style),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Status ", "Статус "), label_style),
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
            Span::styled(
                format!(" {}  ", tr(app.language, "terminate", "завершить")),
                label_style,
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr(app.language, "cancel", "отмена")),
                label_style,
            ),
        ]),
    ];

    let block = Block::default()
        .title(tr(app.language, "Confirm", "Подтверждение"))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_HOT))
        .title_style(title_style);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
