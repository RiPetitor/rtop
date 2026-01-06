use std::fs;
use std::path::{Path, PathBuf};

use ratatui::prelude::*;
use ratatui::style::{Color, Style};

use crate::app::{App, LogoCache, LogoCell, LogoMode, LogoQuality, RenderedLogo};

use super::{ascii, palette, svg};

const LOGO_DIR: &str = "logo";
const ASCII_DIR: &str = "ascii";
const SVG_DIR: &str = "svg";

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

    let needs_render = cache.rendered.as_ref().is_none_or(|rendered| {
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

    cache.palette = palette::load_palette(&root);
    cache.ascii = first_file(&root.join(ASCII_DIR), None).and_then(ascii::load_ascii_logo);
    cache.svg = first_file(&root.join(SVG_DIR), Some("svg")).and_then(svg::load_svg_logo);
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
            .map(|logo| ascii::render_ascii_logo(logo, &cache.palette, quality, width, height))
            .unwrap_or_else(|| RenderedLogo::blank(mode, quality, width, height)),
        LogoMode::Svg => cache
            .svg
            .as_ref()
            .map(|logo| svg::render_svg_logo(logo, quality, width, height))
            .unwrap_or_else(|| RenderedLogo::blank(mode, quality, width, height)),
    }
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
