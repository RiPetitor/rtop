use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, ProcessFilterType};
use crate::ui::text::tr;
use crate::ui::theme::COLOR_MUTED;
use crate::ui::{panel_block, panel_block_focused};
use crate::utils::{fit_text, text_width};

pub fn render_search_panel(frame: &mut Frame, area: Rect, app: &App) {
    let title = tr(app.language, "Process Search", "Поиск процесса");
    let block = if app.process_filter_active {
        panel_block_focused(title)
    } else {
        panel_block(title)
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let filter_type_label = app.process_filter_type.label(app.language);
    let dropdown_indicator = " ▼";

    let label_style = Style::default().fg(COLOR_MUTED);
    let dropdown_style = if app.process_filter_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_MUTED)
    };
    let value_style = if app.process_filter_active || !app.process_filter.is_empty() {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(COLOR_MUTED)
    };

    let value = if app.process_filter_active {
        format!("{}|", app.process_filter)
    } else if app.process_filter.is_empty() {
        let hint = match app.process_filter_type {
            ProcessFilterType::Name => {
                tr(app.language, "press / to search", "нажмите / для поиска")
            }
            ProcessFilterType::Pid => tr(app.language, "enter PID", "введите PID"),
            ProcessFilterType::User => tr(app.language, "enter username", "введите имя"),
        };
        hint.to_string()
    } else {
        app.process_filter.clone()
    };

    let max_width = inner.width as usize;
    let prefix = format!("[{filter_type_label}{dropdown_indicator}]: ");
    let prefix_width = text_width(&prefix);
    let value = if prefix_width < max_width {
        fit_text(&value, max_width - prefix_width)
    } else {
        String::new()
    };

    let line = Line::from(vec![
        Span::styled("[", label_style),
        Span::styled(filter_type_label, dropdown_style),
        Span::styled(dropdown_indicator, dropdown_style),
        Span::styled("]: ", label_style),
        Span::styled(value, value_style),
    ]);
    let paragraph = Paragraph::new(vec![line]);
    frame.render_widget(paragraph, inner);
}
