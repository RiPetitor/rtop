use std::fs;
use std::path::PathBuf;

use crate::app::{
    AsciiCell, AsciiLogo, LogoCell, LogoMode, LogoPalette, LogoQuality, RenderedLogo,
};

pub(super) fn load_ascii_logo(path: PathBuf) -> Option<AsciiLogo> {
    let content = fs::read_to_string(path).ok()?;
    parse_ascii_logo(&content)
}

fn parse_ascii_logo(content: &str) -> Option<AsciiLogo> {
    let mut lines = Vec::new();
    let mut max_width = 0;

    for raw_line in content.lines() {
        let mut row = Vec::new();
        let mut chars = raw_line.chars().peekable();
        let mut current_color: Option<u8> = None;
        while let Some(ch) = chars.next() {
            if ch == '$' {
                match chars.peek().copied() {
                    Some('$') => {
                        chars.next();
                        row.push(AsciiCell {
                            ch: '$',
                            color_index: current_color,
                        });
                    }
                    Some('0'..='9') => {
                        let digit = chars.next().unwrap();
                        current_color = digit.to_digit(10).map(|value| value as u8);
                    }
                    _ => {
                        row.push(AsciiCell {
                            ch,
                            color_index: current_color,
                        });
                    }
                }
                continue;
            }
            row.push(AsciiCell {
                ch,
                color_index: current_color,
            });
        }
        max_width = max_width.max(row.len());
        lines.push(row);
    }

    if lines.is_empty() {
        return None;
    }

    for row in &mut lines {
        if row.len() < max_width {
            row.resize(max_width, AsciiCell::blank());
        }
    }

    let mut min_x = max_width;
    let mut max_x = 0;
    let mut min_y = lines.len();
    let mut max_y = 0;
    let mut found = false;
    for (y, row) in lines.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.ch != ' ' {
                found = true;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    if !found {
        return None;
    }

    let trimmed: Vec<Vec<AsciiCell>> = lines[min_y..=max_y]
        .iter()
        .map(|row| row[min_x..=max_x].to_vec())
        .collect();
    let height = trimmed.len();
    let width = trimmed.first().map(|row| row.len()).unwrap_or(0);
    Some(AsciiLogo {
        width,
        height,
        cells: trimmed,
    })
}

pub(super) fn render_ascii_logo(
    logo: &AsciiLogo,
    palette: &LogoPalette,
    quality: LogoQuality,
    width: u16,
    height: u16,
) -> RenderedLogo {
    let target_w = width as usize;
    let target_h = height as usize;
    let scaled = scale_ascii_logo(logo, target_w, target_h);
    let mut cells = Vec::with_capacity(target_w * target_h);
    for row in scaled {
        for cell in row {
            let fg = cell
                .color_index
                .and_then(|idx| palette.color_for_index(idx));
            cells.push(LogoCell {
                ch: cell.ch,
                fg,
                bg: None,
            });
        }
    }
    RenderedLogo {
        mode: LogoMode::Ascii,
        quality,
        width,
        height,
        cells,
    }
}

fn scale_ascii_logo(logo: &AsciiLogo, target_w: usize, target_h: usize) -> Vec<Vec<AsciiCell>> {
    let mut canvas = vec![vec![AsciiCell::blank(); target_w]; target_h];
    if logo.width == 0 || logo.height == 0 || target_w == 0 || target_h == 0 {
        return canvas;
    }
    let (scaled_w, scaled_h) = fit_dimensions_usize(logo.width, logo.height, target_w, target_h);
    let scaled = scale_ascii_cells(&logo.cells, logo.width, logo.height, scaled_w, scaled_h);
    let offset_x = (target_w.saturating_sub(scaled_w)) / 2;
    let offset_y = (target_h.saturating_sub(scaled_h)) / 2;
    for y in 0..scaled_h {
        for x in 0..scaled_w {
            canvas[y + offset_y][x + offset_x] = scaled[y][x].clone();
        }
    }
    canvas
}

fn scale_ascii_cells(
    src: &[Vec<AsciiCell>],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> Vec<Vec<AsciiCell>> {
    let mut rows = Vec::with_capacity(dst_h);
    for y in 0..dst_h {
        let src_y = y.saturating_mul(src_h) / dst_h.max(1);
        let mut row = Vec::with_capacity(dst_w);
        for x in 0..dst_w {
            let src_x = x.saturating_mul(src_w) / dst_w.max(1);
            row.push(src[src_y][src_x].clone());
        }
        rows.push(row);
    }
    rows
}

fn fit_dimensions_usize(src_w: usize, src_h: usize, max_w: usize, max_h: usize) -> (usize, usize) {
    if src_w == 0 || src_h == 0 || max_w == 0 || max_h == 0 {
        return (0, 0);
    }
    let scale_w = max_w as f32 / src_w as f32;
    let scale_h = max_h as f32 / src_h as f32;
    let scale = scale_w.min(scale_h);
    let new_w = (src_w as f32 * scale).round().max(1.0) as usize;
    let new_h = (src_h as f32 * scale).round().max(1.0) as usize;
    (new_w.min(max_w), new_h.min(max_h))
}
