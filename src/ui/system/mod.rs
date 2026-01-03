mod info;
mod logo;

use ratatui::prelude::*;

use super::panel_block;
use super::text::tr;
use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block(tr(app.language, "System", "Система"));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let selected = logo::select_logo();
    let min_info_width = 24;
    let max_logo_width = inner
        .width
        .saturating_sub(min_info_width)
        .max(10)
        .min(inner.width);
    let logo_width = (logo::logo_max_width(selected).min(max_logo_width as usize) as u16)
        .max(8)
        .min(inner.width);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(logo_width), Constraint::Min(0)])
        .split(inner);

    logo::render_logo(frame, chunks[0], selected);
    info::render_info(frame, chunks[1], app);
}
