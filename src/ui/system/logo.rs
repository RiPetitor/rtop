use std::fs;
use std::path::{Path, PathBuf};

use ratatui::prelude::*;
use ratatui::style::{Color, Style};

use serde::Deserialize;

use resvg::tiny_skia;
use resvg::usvg;

use crate::app::{
    App, AsciiCell, AsciiLogo, LogoCache, LogoCell, LogoMode, LogoPalette, LogoQuality,
    RenderedLogo, RgbColor, RgbaColor, SvgLogo,
};

const LOGO_DIR: &str = "logo";
const ASCII_DIR: &str = "ascii";
const SVG_DIR: &str = "svg";
const PALETTE_JSON: &str = "palette.json";
const PALETTE_YAML: &str = "palette.yaml";
const PALETTE_YML: &str = "palette.yml";
const MAX_SVG_DIM: u32 = 2048;
const ALPHA_THRESHOLD: u8 = 10;

#[derive(Deserialize)]
struct PaletteFile {
    default: Option<[u8; 3]>,
    colors: Option<Vec<[u8; 3]>>,
}

pub fn render_logo(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let preferred = app.logo_mode;
    let quality = app.logo_quality;
    let cache = ensure_logo_cache(app);
    let Some(mode) = select_logo_mode(cache, preferred) else {
        let blank = RenderedLogo::blank(preferred, quality, area.width, area.height);
        draw_rendered_logo(frame, area, &blank);
        return;
    };

    let needs_render = cache.rendered.as_ref().map_or(true, |rendered| {
        rendered.mode != mode
            || rendered.quality != quality
            || rendered.width != area.width
            || rendered.height != area.height
    });
    if needs_render {
        cache.rendered = Some(build_rendered_logo(
            cache,
            mode,
            quality,
            area.width,
            area.height,
        ));
    }

    if let Some(rendered) = cache.rendered.as_ref() {
        draw_rendered_logo(frame, area, rendered);
    }
}

fn ensure_logo_cache(app: &mut App) -> &mut LogoCache {
    if app.logo_cache.is_none() {
        app.logo_cache = Some(load_logo_cache());
    }
    app.logo_cache.as_mut().expect("logo cache initialized")
}

fn select_logo_mode(cache: &LogoCache, preferred: LogoMode) -> Option<LogoMode> {
    let has_ascii = cache.ascii.is_some();
    let has_svg = cache.svg.is_some();
    match preferred {
        LogoMode::Ascii if has_ascii => Some(LogoMode::Ascii),
        LogoMode::Svg if has_svg => Some(LogoMode::Svg),
        LogoMode::Ascii if has_svg => Some(LogoMode::Svg),
        LogoMode::Svg if has_ascii => Some(LogoMode::Ascii),
        _ => None,
    }
}

fn load_logo_cache() -> LogoCache {
    let mut cache = LogoCache::default();
    let Some(root) = logo_root() else {
        return cache;
    };

    cache.palette = load_palette(&root);
    cache.ascii = first_file(&root.join(ASCII_DIR), None).and_then(load_ascii_logo);
    cache.svg = first_file(&root.join(SVG_DIR), Some("svg")).and_then(load_svg_logo);
    cache
}

fn logo_root() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("rtop").join(LOGO_DIR))
}

fn first_file(dir: &Path, extension: Option<&str>) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            !name.starts_with('.')
        })
        .filter(|path| match extension {
            Some(ext) => path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case(ext))
                .unwrap_or(false),
            None => true,
        })
        .collect();
    files.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    files.into_iter().next()
}

fn load_palette(root: &Path) -> LogoPalette {
    let json_path = root.join(PALETTE_JSON);
    if let Some(palette) = load_palette_json(&json_path) {
        return palette;
    }

    let yaml_path = root.join(PALETTE_YAML);
    if let Some(palette) = load_palette_yaml(&yaml_path) {
        return palette;
    }

    let yml_path = root.join(PALETTE_YML);
    if let Some(palette) = load_palette_yaml(&yml_path) {
        return palette;
    }

    LogoPalette::default()
}

fn load_palette_json(path: &Path) -> Option<LogoPalette> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: PaletteFile = serde_json::from_str(&content).ok()?;
    Some(palette_from_file(parsed))
}

fn load_palette_yaml(path: &Path) -> Option<LogoPalette> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: PaletteFile = serde_yaml_ng::from_str(&content).ok()?;
    Some(palette_from_file(parsed))
}

fn palette_from_file(file: PaletteFile) -> LogoPalette {
    let mut palette = LogoPalette::default();
    if let Some(default) = file.default {
        palette.default = Some(RgbColor {
            r: default[0],
            g: default[1],
            b: default[2],
        });
    }
    if let Some(colors) = file.colors {
        palette.colors = colors
            .into_iter()
            .map(|rgb| RgbColor {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
            })
            .collect();
    }
    palette
}

fn load_ascii_logo(path: PathBuf) -> Option<AsciiLogo> {
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
                        continue;
                    }
                    Some(next) if next.is_ascii_digit() => {
                        let digit = next.to_digit(10).unwrap_or(0) as u8;
                        chars.next();
                        if digit == 0 {
                            current_color = None;
                        } else {
                            current_color = Some(digit);
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            row.push(AsciiCell {
                ch,
                color_index: current_color,
            });
        }
        max_width = max_width.max(row.len());
        lines.push(row);
    }

    if lines.is_empty() || max_width == 0 {
        return None;
    }

    for row in &mut lines {
        while row.len() < max_width {
            row.push(AsciiCell::blank());
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

fn load_svg_logo(path: PathBuf) -> Option<SvgLogo> {
    let data = fs::read(&path).ok()?;
    let mut options = usvg::Options::default();
    options.resources_dir = path.parent().map(|dir| dir.to_path_buf());
    let tree = usvg::Tree::from_data(&data, &options).ok()?;
    Some(SvgLogo { tree })
}

fn render_svg_pixels(tree: &usvg::Tree, width: u32, height: u32) -> Option<Vec<RgbaColor>> {
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    let size = tree.size();
    let scale_x = if size.width() > 0.0 {
        width as f32 / size.width()
    } else {
        1.0
    };
    let scale_y = if size.height() > 0.0 {
        height as f32 / size.height()
    } else {
        1.0
    };
    let scale = scale_x.min(scale_y);
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    Some(pixmap_to_pixels(&pixmap))
}

fn pixmap_to_pixels(pixmap: &tiny_skia::Pixmap) -> Vec<RgbaColor> {
    let data = pixmap.data();
    let mut pixels = Vec::with_capacity((pixmap.width() * pixmap.height()) as usize);
    for chunk in data.chunks_exact(4) {
        pixels.push(RgbaColor {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: chunk[3],
        });
    }
    pixels
}

fn crop_pixels(
    pixels: &[RgbaColor],
    width: u32,
    height: u32,
) -> Option<(u32, u32, Vec<RgbaColor>)> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    if pixels.len() < width_usize.saturating_mul(height_usize) {
        return None;
    }

    let mut min_x = width_usize;
    let mut max_x = 0;
    let mut min_y = height_usize;
    let mut max_y = 0;
    let mut found = false;
    for y in 0..height_usize {
        for x in 0..width_usize {
            let idx = y * width_usize + x;
            if pixels[idx].a > ALPHA_THRESHOLD {
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

    let cropped_width = max_x - min_x + 1;
    let cropped_height = max_y - min_y + 1;
    let mut cropped = Vec::with_capacity(cropped_width * cropped_height);
    for y in min_y..=max_y {
        let row_start = y * width_usize;
        for x in min_x..=max_x {
            cropped.push(pixels[row_start + x]);
        }
    }

    Some((cropped_width as u32, cropped_height as u32, cropped))
}

fn build_rendered_logo(
    cache: &LogoCache,
    mode: LogoMode,
    quality: LogoQuality,
    width: u16,
    height: u16,
) -> RenderedLogo {
    match mode {
        LogoMode::Ascii => cache
            .ascii
            .as_ref()
            .map(|logo| render_ascii_logo(logo, &cache.palette, quality, width, height))
            .unwrap_or_else(|| RenderedLogo::blank(mode, quality, width, height)),
        LogoMode::Svg => cache
            .svg
            .as_ref()
            .map(|logo| render_svg_logo(logo, quality, width, height))
            .unwrap_or_else(|| RenderedLogo::blank(mode, quality, width, height)),
    }
}

fn render_ascii_logo(
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

fn render_svg_logo(logo: &SvgLogo, quality: LogoQuality, width: u16, height: u16) -> RenderedLogo {
    let target_w = u32::from(width);
    let target_h = u32::from(height).saturating_mul(2);
    if target_w == 0 || target_h == 0 {
        return RenderedLogo::blank(LogoMode::Svg, quality, width, height);
    }

    let desired_scale = quality.scale();
    let effective_scale = effective_svg_scale(target_w, target_h, desired_scale);
    let hi_w = target_w.saturating_mul(effective_scale);
    let hi_h = target_h.saturating_mul(effective_scale);
    if hi_w == 0 || hi_h == 0 {
        return RenderedLogo::blank(LogoMode::Svg, quality, width, height);
    }

    let Some(pixels) = render_svg_pixels(&logo.tree, hi_w, hi_h) else {
        return RenderedLogo::blank(LogoMode::Svg, quality, width, height);
    };
    let Some((crop_w, crop_h, cropped)) = crop_pixels(&pixels, hi_w, hi_h) else {
        return RenderedLogo::blank(LogoMode::Svg, quality, width, height);
    };

    let (scaled_w, scaled_h) = fit_dimensions_u32(crop_w, crop_h, hi_w, hi_h);
    let scaled = if scaled_w != crop_w || scaled_h != crop_h {
        scale_pixels_nearest(&cropped, crop_w, crop_h, scaled_w, scaled_h)
    } else {
        cropped
    };
    let mut canvas = vec![RgbaColor::default(); (hi_w * hi_h) as usize];
    let offset_x = (hi_w.saturating_sub(scaled_w)) / 2;
    let offset_y = (hi_h.saturating_sub(scaled_h)) / 2;
    for y in 0..scaled_h {
        for x in 0..scaled_w {
            let src_idx = (y * scaled_w + x) as usize;
            let dst_idx = ((y + offset_y) * hi_w + x + offset_x) as usize;
            canvas[dst_idx] = scaled[src_idx];
        }
    }

    let pixels = if effective_scale > 1 {
        downsample_pixels_box(&canvas, hi_w, hi_h, effective_scale)
    } else {
        canvas
    };

    let mut cells = Vec::with_capacity(width as usize * height as usize);
    for cell_y in 0..height {
        let top_row = u32::from(cell_y).saturating_mul(2);
        let bottom_row = top_row + 1;
        for cell_x in 0..width {
            let col = u32::from(cell_x);
            let top = pixels[(top_row * target_w + col) as usize];
            let bottom = if bottom_row < target_h {
                pixels[(bottom_row * target_w + col) as usize]
            } else {
                RgbaColor::default()
            };
            cells.push(svg_cell_from_pixels(top, bottom));
        }
    }

    RenderedLogo {
        mode: LogoMode::Svg,
        quality,
        width,
        height,
        cells,
    }
}

fn effective_svg_scale(target_w: u32, target_h: u32, desired_scale: u32) -> u32 {
    let max_scale_w = MAX_SVG_DIM / target_w.max(1);
    let max_scale_h = MAX_SVG_DIM / target_h.max(1);
    let max_scale = max_scale_w.min(max_scale_h).max(1);
    desired_scale.min(max_scale).max(1)
}

fn scale_pixels_nearest(
    src: &[RgbaColor],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Vec<RgbaColor> {
    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        return Vec::new();
    }
    let mut out = vec![RgbaColor::default(); (dst_w * dst_h) as usize];
    for y in 0..dst_h {
        let src_y = y.saturating_mul(src_h) / dst_h.max(1);
        for x in 0..dst_w {
            let src_x = x.saturating_mul(src_w) / dst_w.max(1);
            let src_idx = (src_y * src_w + src_x) as usize;
            let dst_idx = (y * dst_w + x) as usize;
            out[dst_idx] = src[src_idx];
        }
    }
    out
}

fn downsample_pixels_box(src: &[RgbaColor], src_w: u32, src_h: u32, scale: u32) -> Vec<RgbaColor> {
    if scale <= 1 || src_w == 0 || src_h == 0 {
        return src.to_vec();
    }
    let dst_w = src_w / scale;
    let dst_h = src_h / scale;
    let mut out = vec![RgbaColor::default(); (dst_w * dst_h) as usize];
    let block = (scale * scale) as u64;
    for y in 0..dst_h {
        for x in 0..dst_w {
            let mut sum_r: u64 = 0;
            let mut sum_g: u64 = 0;
            let mut sum_b: u64 = 0;
            let mut sum_a: u64 = 0;
            for by in 0..scale {
                for bx in 0..scale {
                    let src_x = x * scale + bx;
                    let src_y = y * scale + by;
                    let idx = (src_y * src_w + src_x) as usize;
                    let px = src[idx];
                    let a = px.a as u64;
                    sum_a += a;
                    sum_r += (px.r as u64) * a;
                    sum_g += (px.g as u64) * a;
                    sum_b += (px.b as u64) * a;
                }
            }
            let dst_idx = (y * dst_w + x) as usize;
            if sum_a == 0 {
                out[dst_idx] = RgbaColor::default();
            } else {
                let avg_a = (sum_a / block) as u8;
                out[dst_idx] = RgbaColor {
                    r: (sum_r / sum_a) as u8,
                    g: (sum_g / sum_a) as u8,
                    b: (sum_b / sum_a) as u8,
                    a: avg_a,
                };
            }
        }
    }
    out
}

fn svg_cell_from_pixels(top: RgbaColor, bottom: RgbaColor) -> LogoCell {
    let top_visible = top.a > ALPHA_THRESHOLD;
    let bottom_visible = bottom.a > ALPHA_THRESHOLD;
    match (top_visible, bottom_visible) {
        (false, false) => LogoCell::blank(),
        (true, false) => LogoCell {
            ch: '▀',
            fg: Some(rgba_to_rgb(top)),
            bg: None,
        },
        (false, true) => LogoCell {
            ch: '▄',
            fg: Some(rgba_to_rgb(bottom)),
            bg: None,
        },
        (true, true) => LogoCell {
            ch: '▀',
            fg: Some(rgba_to_rgb(top)),
            bg: Some(rgba_to_rgb(bottom)),
        },
    }
}

fn rgba_to_rgb(color: RgbaColor) -> RgbColor {
    RgbColor {
        r: color.r,
        g: color.g,
        b: color.b,
    }
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

fn fit_dimensions_u32(src_w: u32, src_h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if src_w == 0 || src_h == 0 || max_w == 0 || max_h == 0 {
        return (0, 0);
    }
    let scale_w = max_w as f32 / src_w as f32;
    let scale_h = max_h as f32 / src_h as f32;
    let scale = scale_w.min(scale_h);
    let new_w = (src_w as f32 * scale).round().max(1.0) as u32;
    let new_h = (src_h as f32 * scale).round().max(1.0) as u32;
    (new_w.min(max_w), new_h.min(max_h))
}

fn draw_rendered_logo(frame: &mut Frame, area: Rect, rendered: &RenderedLogo) {
    let buffer = frame.buffer_mut();
    for y in 0..area.height {
        for x in 0..area.width {
            let idx = (y as usize) * rendered.width as usize + x as usize;
            let cell = rendered
                .cells
                .get(idx)
                .copied()
                .unwrap_or_else(LogoCell::blank);
            let mut style = Style::default();
            if let Some(fg) = cell.fg {
                style = style.fg(Color::Rgb(fg.r, fg.g, fg.b));
            }
            if let Some(bg) = cell.bg {
                style = style.bg(Color::Rgb(bg.r, bg.g, bg.b));
            }
            if let Some(buf_cell) = buffer.cell_mut((area.x + x, area.y + y)) {
                let mut symbol_buf = [0u8; 4];
                let symbol = cell.ch.encode_utf8(&mut symbol_buf);
                buf_cell.set_symbol(symbol);
                buf_cell.set_style(style);
            }
        }
    }
}
