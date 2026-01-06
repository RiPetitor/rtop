use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn text_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

pub fn format_bytes(bytes: u64) -> String {
    const UNIT: f64 = 1024.0;
    let bytes = bytes as f64;

    if bytes < UNIT {
        return format!("{bytes:.0} B");
    }

    let kb = bytes / UNIT;
    if kb < UNIT {
        return format!("{kb:.1} KiB");
    }

    let mb = kb / UNIT;
    if mb < UNIT {
        return format!("{mb:.1} MiB");
    }

    let gb = mb / UNIT;
    if gb < UNIT {
        return format!("{gb:.1} GiB");
    }

    let tb = gb / UNIT;
    format!("{tb:.1} TiB")
}

pub fn mib_to_bytes(mib: u64) -> u64 {
    mib.saturating_mul(1024 * 1024)
}

pub fn fit_text(value: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if text_width(value) <= max_len {
        return value.to_string();
    }
    if max_len <= 3 {
        return take_width(value, max_len);
    }
    let mut trimmed = take_width(value, max_len - 3);
    trimmed.push_str("...");
    trimmed
}

pub fn take_width(value: &str, max_len: usize) -> String {
    let mut output = String::new();
    let mut width = 0;
    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_len {
            break;
        }
        output.push(ch);
        width += ch_width;
    }
    output
}

pub fn format_duration(secs: u64) -> String {
    let mut remaining = secs;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;

    if days > 0 {
        format!("{days}d {hours:02}h {minutes:02}m")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m {seconds:02}s")
    }
}

pub fn format_duration_short(secs: u64) -> String {
    let mut remaining = secs;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;

    if days > 0 {
        format!("{days}d {hours:02}h")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m {seconds:02}s")
    }
}

pub fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f32 / total as f32) * 100.0
    }
}

pub fn render_bar(pct: f32, width: usize) -> String {
    let width = width.max(1);
    let pct = if pct.is_finite() {
        pct.clamp(0.0, 100.0)
    } else {
        0.0
    };
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_minutes_seconds() {
        assert_eq!(format_duration(75), "1m 15s");
    }

    #[test]
    fn format_duration_hours_minutes() {
        assert_eq!(format_duration(3661), "1h 01m");
        assert_eq!(format_duration(7200), "2h 00m");
    }

    #[test]
    fn format_duration_days_hours_minutes() {
        assert_eq!(format_duration(90061), "1d 01h 01m");
        assert_eq!(format_duration(172800), "2d 00h 00m");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0m 00s");
    }

    #[test]
    fn format_duration_short_minutes_seconds() {
        assert_eq!(format_duration_short(75), "1m 15s");
    }

    #[test]
    fn format_duration_short_hours_minutes() {
        assert_eq!(format_duration_short(3661), "1h 01m");
    }

    #[test]
    fn format_duration_short_days_hours() {
        assert_eq!(format_duration_short(90061), "1d 01h");
    }

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn format_bytes_fractional() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(1536 * 1024), "1.5 MiB");
    }

    #[test]
    fn format_bytes_large() {
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.0 TiB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024 * 1024), "2.0 TiB");
    }

    #[test]
    fn mib_to_bytes_conversion() {
        assert_eq!(mib_to_bytes(0), 0);
        assert_eq!(mib_to_bytes(1), 1024 * 1024);
        assert_eq!(mib_to_bytes(1024), 1024 * 1024 * 1024);
    }

    #[test]
    fn mib_to_bytes_overflow() {
        let result = mib_to_bytes(u64::MAX / (1024 * 1024) + 1);
        assert_eq!(result, u64::MAX);
    }

    #[test]
    fn text_width_ascii() {
        assert_eq!(text_width("hello"), 5);
        assert_eq!(text_width(""), 0);
    }

    #[test]
    fn text_width_counts_display_cells() {
        assert_eq!(text_width("Не найдено"), 10);
        assert_eq!(text_width("表"), 2);
        assert_eq!(text_width("a表b"), 4);
    }

    #[test]
    fn take_width_ascii() {
        assert_eq!(take_width("hello", 3), "hel");
        assert_eq!(take_width("hello", 10), "hello");
    }

    #[test]
    fn take_width_wide_chars() {
        assert_eq!(take_width("表表表", 3), "表");
        assert_eq!(take_width("表表表", 4), "表表");
    }

    #[test]
    fn take_width_empty() {
        assert_eq!(take_width("", 5), "");
    }

    #[test]
    fn fit_text_no_trim() {
        assert_eq!(fit_text("hello", 10), "hello");
        assert_eq!(fit_text("", 5), "");
    }

    #[test]
    fn fit_text_trim() {
        assert_eq!(fit_text("hello world", 8), "hello...");
    }

    #[test]
    fn fit_text_trims_by_display_width() {
        assert_eq!(fit_text("表表表", 5), "表...");
    }

    #[test]
    fn fit_text_zero_max_len() {
        assert_eq!(fit_text("hello", 0), "");
    }

    #[test]
    fn fit_text_small_max_len() {
        assert_eq!(fit_text("hello", 2), "he");
        assert_eq!(fit_text("hello", 3), "hel");
    }

    #[test]
    fn percent_normal() {
        assert_eq!(percent(50, 100), 50.0);
        assert_eq!(percent(1, 4), 25.0);
    }

    #[test]
    fn percent_zero_total() {
        assert_eq!(percent(100, 0), 0.0);
    }

    #[test]
    fn percent_fractional() {
        let result = percent(1, 3);
        assert!((result - 33.333332).abs() < 0.01);
    }

    #[test]
    fn percent_over_100() {
        assert_eq!(percent(150, 100), 150.0);
    }

    #[test]
    fn render_bar_empty() {
        assert_eq!(render_bar(0.0, 10), "░░░░░░░░░░");
    }

    #[test]
    fn render_bar_full() {
        assert_eq!(render_bar(100.0, 10), "██████████");
    }

    #[test]
    fn render_bar_half() {
        assert_eq!(render_bar(50.0, 10), "█████░░░░░");
    }

    #[test]
    fn render_bar_fractional() {
        let result = render_bar(33.33, 10);
        assert!(result.starts_with("███"));
        assert!(result.contains("░"));
    }

    #[test]
    fn render_bar_clamped() {
        assert_eq!(render_bar(-10.0, 10), "░░░░░░░░░░");
        assert_eq!(render_bar(150.0, 10), "██████████");
    }

    #[test]
    fn render_bar_nan() {
        assert_eq!(render_bar(f32::NAN, 10), "░░░░░░░░░░");
    }

    #[test]
    fn render_bar_infinity() {
        assert_eq!(render_bar(f32::INFINITY, 10), "░░░░░░░░░░");
        assert_eq!(render_bar(f32::NEG_INFINITY, 10), "░░░░░░░░░░");
    }

    #[test]
    fn render_bar_min_width() {
        assert_eq!(render_bar(100.0, 1), "█");
        assert_eq!(render_bar(0.0, 1), "░");
    }
}
