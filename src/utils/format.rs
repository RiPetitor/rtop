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
    format!("{}{}", "=".repeat(filled), ".".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_minutes_seconds() {
        assert_eq!(format_duration(75), "1m 15s");
    }

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn text_width_counts_display_cells() {
        assert_eq!(text_width("Не найдено"), 10);
        assert_eq!(text_width("表"), 2);
    }

    #[test]
    fn fit_text_trims_by_display_width() {
        assert_eq!(fit_text("表表表", 5), "表...");
    }
}
