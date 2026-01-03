use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    if !app.show_help {
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

    let lines = vec![
        Line::from(Span::styled(
            tr(app.language, "Quick Keys", "Быстрые клавиши"),
            label_style,
        )),
        Line::from(vec![
            Span::styled("F2", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "setup", "настройки")),
                hint_style,
            ),
            Span::styled("F12", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "help", "справка")),
                hint_style,
            ),
            Span::styled("q/й", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "quit", "выход")),
                hint_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            tr(app.language, "Navigation", "Навигация"),
            label_style,
        )),
        Line::from(vec![
            Span::styled("Up/Down", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "move", "перемещение")),
                hint_style,
            ),
            Span::styled("Enter", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "action", "действие")),
                hint_style,
            ),
            Span::styled("Esc", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "back/close", "назад/закрыть")),
                hint_style,
            ),
        ]),
        Line::from(vec![
            Span::styled("Home/End", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "top/bottom", "вверх/вниз")),
                hint_style,
            ),
            Span::styled("PgUp/PgDn", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "page", "страница")),
                hint_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            tr(app.language, "Sorting", "Сортировка"),
            label_style,
        )),
        Line::from(vec![
            Span::styled("Left/Right", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "column", "колонка")),
                hint_style,
            ),
            Span::styled("Space", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "order", "порядок")),
                hint_style,
            ),
            Span::styled("Mouse", key_style),
            Span::styled(
                format!(
                    " {}",
                    tr(app.language, "header sort", "сортировка по заголовку")
                ),
                hint_style,
            ),
        ]),
        Line::from(vec![
            Span::styled("c/m/p/n/u (с/ь/з/т/г)", key_style),
            Span::styled(
                format!(
                    " {}  ",
                    tr(app.language, "quick sort", "быстрая сортировка")
                ),
                hint_style,
            ),
            Span::styled("h/р", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "highlight", "подсветка")),
                hint_style,
            ),
        ]),
        Line::from(vec![
            Span::styled(tr(app.language, "Tree mode", "Дерево"), label_style),
            Span::styled(
                format!(" {}", tr(app.language, "PID order only", "только PID")),
                hint_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            tr(app.language, "Views", "Режимы"),
            label_style,
        )),
        Line::from(vec![
            Span::styled("1", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "overview", "обзор")),
                hint_style,
            ),
            Span::styled("2", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "system", "система")),
                hint_style,
            ),
            Span::styled("3", key_style),
            Span::styled(format!(" {}  ", tr(app.language, "gpu", "gpu")), hint_style),
            Span::styled("4", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "containers", "контейнеры")),
                hint_style,
            ),
            Span::styled("Tab", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "cycle", "цикл")),
                hint_style,
            ),
        ]),
        Line::from(vec![
            Span::styled("t/е", key_style),
            Span::styled(
                format!(
                    " {}",
                    tr(app.language, "tree (Processes)", "дерево (Процессы)")
                ),
                hint_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(tr(app.language, "GPU", "GPU"), label_style)),
        Line::from(vec![
            Span::styled("g/G (п/П)", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "select GPU", "выбор GPU")),
                hint_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            tr(app.language, "Other", "Прочее"),
            label_style,
        )),
        Line::from(vec![
            Span::styled("r/к", key_style),
            Span::styled(
                format!(" {}  ", tr(app.language, "refresh", "обновить")),
                hint_style,
            ),
            Span::styled("b/и", key_style),
            Span::styled(
                format!(" {}", tr(app.language, "back", "назад")),
                hint_style,
            ),
        ]),
    ];

    let block = Block::default()
        .title(tr(app.language, "Help", "Справка"))
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
