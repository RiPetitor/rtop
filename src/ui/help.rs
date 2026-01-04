use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    if !app.show_help {
        return;
    }

    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let key_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(COLOR_MUTED);

    // Fixed column widths for alignment
    let col1 = 14; // Key column
    let col2 = 24; // Description column

    let mut lines = Vec::new();

    // Section: Quick Keys
    lines.push(Line::from(Span::styled(
        tr(app.language, "Quick Keys", "Быстрые клавиши"),
        label_style,
    )));
    lines.push(make_row(
        "F2",
        tr(app.language, "Setup", "Настройки"),
        "F12",
        tr(app.language, "Help", "Справка"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "q/й",
        tr(app.language, "Quit", "Выход"),
        "r/к",
        tr(app.language, "Refresh", "Обновить"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(Line::from(""));

    // Section: Navigation
    lines.push(Line::from(Span::styled(
        tr(app.language, "Navigation", "Навигация"),
        label_style,
    )));
    lines.push(make_row(
        "↑/↓",
        tr(app.language, "Move selection", "Перемещение"),
        "Enter",
        tr(app.language, "Expand/Kill", "Развернуть/Убить"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "Home/End",
        tr(app.language, "First/Last", "Начало/Конец"),
        "PgUp/PgDn",
        tr(app.language, "Page up/down", "Страница"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "Esc/b/и",
        tr(app.language, "Back/Close", "Назад/Закрыть"),
        "Tab",
        tr(app.language, "Switch panel/tab", "Панель/вкладка"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(Line::from(""));

    // Section: Sorting
    lines.push(Line::from(Span::styled(
        tr(app.language, "Sorting", "Сортировка"),
        label_style,
    )));
    lines.push(make_row(
        "←/→",
        tr(app.language, "Change column/tab", "Колонка/вкладка"),
        "Space",
        tr(app.language, "Toggle order", "Изменить порядок"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "c/с",
        tr(app.language, "Sort by CPU", "По CPU"),
        "m/ь",
        tr(app.language, "Sort by Memory", "По памяти"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "p/з",
        tr(app.language, "Sort by PID", "По PID"),
        "n/т",
        tr(app.language, "Sort by Name", "По имени"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "u/г",
        tr(app.language, "Sort by User", "По юзеру"),
        "h/р",
        tr(app.language, "Highlight mode", "Режим подсветки"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(Line::from(""));

    // Section: Views
    lines.push(Line::from(Span::styled(
        tr(app.language, "Views", "Режимы"),
        label_style,
    )));
    lines.push(make_row(
        "1",
        tr(app.language, "Overview", "Обзор"),
        "2",
        tr(app.language, "System Info", "Система"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "3",
        tr(app.language, "GPU", "GPU"),
        "4",
        tr(app.language, "Containers", "Контейнеры"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(make_row(
        "5",
        tr(app.language, "Processes", "Процессы"),
        "t/е",
        tr(app.language, "Tree view", "Дерево"),
        col1,
        col2,
        key_style,
        hint_style,
    ));
    lines.push(Line::from(""));

    // Section: GPU
    lines.push(Line::from(Span::styled(
        tr(app.language, "GPU", "GPU"),
        label_style,
    )));
    lines.push(make_row(
        "g/п",
        tr(app.language, "Next GPU", "Следующий GPU"),
        "G/П",
        tr(app.language, "Previous GPU", "Предыдущий GPU"),
        col1,
        col2,
        key_style,
        hint_style,
    ));

    let block = Block::default()
        .title(tr(app.language, " Help ", " Справка "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .title_style(
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}

fn make_row(
    key1: &str,
    desc1: &str,
    key2: &str,
    desc2: &str,
    col1: usize,
    col2: usize,
    key_style: Style,
    hint_style: Style,
) -> Line<'static> {
    let key1_padded = format!("{:<width$}", key1, width = col1);
    let desc1_padded = format!("{:<width$}", desc1, width = col2);
    let key2_padded = format!("{:<width$}", key2, width = col1);

    Line::from(vec![
        Span::styled(key1_padded, key_style),
        Span::styled(desc1_padded, hint_style),
        Span::styled(key2_padded, key_style),
        Span::styled(desc2.to_string(), hint_style),
    ])
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
