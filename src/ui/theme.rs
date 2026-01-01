use ratatui::style::Color;

pub const COLOR_ACCENT: Color = Color::Rgb(78, 190, 210);
pub const COLOR_MUTED: Color = Color::Rgb(138, 148, 158);
pub const COLOR_BORDER: Color = Color::Rgb(78, 86, 96);
pub const COLOR_GOOD: Color = Color::Rgb(95, 200, 120);
pub const COLOR_WARN: Color = Color::Rgb(230, 180, 70);
pub const COLOR_HOT: Color = Color::Rgb(230, 90, 70);

pub fn color_for_percent(pct: f32) -> Color {
    if pct < 50.0 {
        COLOR_GOOD
    } else if pct < 80.0 {
        COLOR_WARN
    } else {
        COLOR_HOT
    }
}
