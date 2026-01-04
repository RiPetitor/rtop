mod info;
mod logo;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

use super::text::tr;
use crate::app::{App, SystemTab, SystemTabRegion};
use crate::ui::theme::{COLOR_ACCENT, COLOR_BORDER, COLOR_MUTED};
use crate::utils::{fit_text, text_width};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = system_block(app, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if inner.height <= 1 {
        return;
    }

    let content_area = Rect {
        x: inner.x,
        y: inner.y.saturating_add(1),
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };
    if app.system_tab == SystemTab::Overview {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_area);
        logo::render_logo(frame, chunks[0], app);
        let info_area = Rect {
            x: chunks[1].x.saturating_add(1),
            y: chunks[1].y.saturating_add(1),
            width: chunks[1].width.saturating_sub(1),
            height: chunks[1].height.saturating_sub(1),
        };
        info::render_info(frame, info_area, app);
    } else {
        info::render_info(frame, content_area, app);
    }
}

fn system_block(app: &mut App, area: Rect) -> Block<'static> {
    let title = system_title_line(app, area);
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(COLOR_BORDER))
}

fn system_title_line(app: &mut App, area: Rect) -> Line<'static> {
    let inner_width = area.width.saturating_sub(2) as usize;
    if inner_width == 0 {
        return Line::from("");
    }

    let title_style = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let active_style = title_style;
    let inactive_style = Style::default().fg(COLOR_MUTED);
    let separator_style = Style::default().fg(COLOR_MUTED);
    let update_style = Style::default().fg(COLOR_ACCENT);
    let title_label = tr(app.language, "System", "Система");
    let mut spans = Vec::new();

    let title_text = format!(" {title_label} ");
    let title_text = fit_text(&title_text, inner_width);
    let title_width = text_width(&title_text);
    spans.push(Span::styled(title_text, title_style));

    let mut used = title_width;
    let separator = " | ";
    let separator_width = text_width(separator);
    if used + separator_width > inner_width {
        return Line::from(spans);
    }
    spans.push(Span::styled(separator.to_string(), separator_style));
    used += separator_width;

    let tabs = [
        SystemTab::Overview,
        SystemTab::Cpu,
        SystemTab::Memory,
        SystemTab::Disks,
        SystemTab::Network,
        SystemTab::Temps,
    ];
    let mut x = area.x.saturating_add(1).saturating_add(used as u16);
    for tab in tabs {
        let label = tab_label(tab, app.language);
        let text = format!(" {label} ");
        let tab_width = text_width(&text);
        if used + tab_width > inner_width {
            break;
        }
        let style = if tab == app.system_tab {
            active_style
        } else {
            inactive_style
        };
        spans.push(Span::styled(text, style));
        app.system_tab_regions.push(SystemTabRegion {
            tab,
            rect: Rect {
                x,
                y: area.y,
                width: tab_width as u16,
                height: 1,
            },
        });
        used += tab_width;
        x = x.saturating_add(tab_width as u16);
    }

    let update_label = tr(app.language, "Update", "Обновить");
    let update_text = format!(" {update_label} ");
    let update_width = text_width(&update_text);
    if used + update_width < inner_width {
        let spacer_width = inner_width.saturating_sub(used + update_width);
        if spacer_width > 0 {
            spans.push(Span::raw(" ".repeat(spacer_width)));
            spans.push(Span::styled(update_text, update_style));
            app.system_update_region = Some(Rect {
                x: area
                    .x
                    .saturating_add(1)
                    .saturating_add(used as u16)
                    .saturating_add(spacer_width as u16),
                y: area.y,
                width: update_width as u16,
                height: 1,
            });
        }
    }

    Line::from(spans)
}

fn tab_label(tab: SystemTab, language: crate::app::Language) -> &'static str {
    match tab {
        SystemTab::Overview => tr(language, "Main", "Основное"),
        SystemTab::Cpu => tr(language, "CPU", "CPU"),
        SystemTab::Memory => tr(language, "Mem", "Пам"),
        SystemTab::Disks => tr(language, "Disk", "Диск"),
        SystemTab::Network => tr(language, "Net", "Сеть"),
        SystemTab::Temps => tr(language, "Temp", "Темп"),
    }
}
