use std::fs;
use std::path::PathBuf;

use resvg::tiny_skia;
use resvg::usvg;

use crate::app::{LogoCell, LogoMode, LogoQuality, RenderedLogo, RgbColor, RgbaColor, SvgLogo};

const MAX_SVG_DIM: u32 = 2048;
const ALPHA_THRESHOLD: u8 = 10;

pub(super) fn load_svg_logo(path: PathBuf) -> Option<SvgLogo> {
    let data = fs::read(&path).ok()?;
    let options = usvg::Options {
        resources_dir: path.parent().map(|dir| dir.to_path_buf()),
        ..Default::default()
    };
    let tree = usvg::Tree::from_data(&data, &options).ok()?;
    Some(SvgLogo { tree })
}

pub(super) fn render_svg_logo(
    logo: &SvgLogo,
    quality: LogoQuality,
    width: u16,
    height: u16,
) -> RenderedLogo {
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
