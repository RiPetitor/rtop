use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::app::{LogoPalette, RgbColor};

const PALETTE_JSON: &str = "palette.json";
const PALETTE_YAML: &str = "palette.yaml";
const PALETTE_YML: &str = "palette.yml";

#[derive(Deserialize)]
struct PaletteFile {
    default: Option<[u8; 3]>,
    colors: Option<Vec<[u8; 3]>>,
}

pub(super) fn load_palette(root: &Path) -> LogoPalette {
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
