use ratatui::prelude::Style;
use ratatui::text::{Line, Span};

use crate::app::IconMode;
use crate::utils::{fit_text, text_width};

use super::icons::{ICON_SEP_NERD, IconLabel};

pub(super) fn push_header(lines: &mut Vec<Line<'static>>, title: &str, width: usize, style: Style) {
    let title = fit_text(title, width);
    lines.push(Line::from(Span::styled(title, style)));
}

pub(super) fn push_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: String,
    width: usize,
    label_width: usize,
    label_style: Style,
    value_style: Style,
) {
    let label = pad_label(label, label_width);
    let max_value = width.saturating_sub(text_width(&label)).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(label, label_style),
        Span::styled(value, value_style),
    ]));
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_icon_line(
    lines: &mut Vec<Line<'static>>,
    icon: &IconLabel,
    value: String,
    width: usize,
    icon_style: Style,
    sep_style: Style,
    value_style: Style,
    icon_mode: IconMode,
) {
    let label = icon.get(icon_mode);
    let (icon_text, sep_text) = match icon_mode {
        IconMode::Nerd => (format!("{label} "), format!("{ICON_SEP_NERD} ")),
        IconMode::Text => (format!("{label} "), String::new()),
    };
    let used = text_width(&icon_text) + text_width(&sep_text);
    let max_value = width.saturating_sub(used).max(1);
    let value = fit_text(&value, max_value);
    if sep_text.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(icon_text, icon_style),
            Span::styled(value, value_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(icon_text, icon_style),
            Span::styled(sep_text, sep_style),
            Span::styled(value, value_style),
        ]));
    }
}

fn pad_label(label: &str, width: usize) -> String {
    let trimmed = fit_text(label, width);
    let pad = width.saturating_sub(text_width(&trimmed));
    if pad == 0 {
        trimmed
    } else {
        format!("{trimmed}{}", " ".repeat(pad))
    }
}
