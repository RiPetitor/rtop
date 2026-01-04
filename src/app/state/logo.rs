use resvg::usvg;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogoMode {
    #[default]
    Ascii,
    Svg,
}

impl LogoMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ascii" => Some(LogoMode::Ascii),
            "svg" => Some(LogoMode::Svg),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            LogoMode::Ascii => "ascii",
            LogoMode::Svg => "svg",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            LogoMode::Ascii => LogoMode::Svg,
            LogoMode::Svg => LogoMode::Ascii,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogoQuality {
    Quality,
    #[default]
    Medium,
    Pixel,
}

impl LogoQuality {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "quality" | "high" | "hq" => Some(LogoQuality::Quality),
            "medium" | "med" => Some(LogoQuality::Medium),
            "pixel" | "pix" | "low" => Some(LogoQuality::Pixel),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            LogoQuality::Quality => "quality",
            LogoQuality::Medium => "medium",
            LogoQuality::Pixel => "pixel",
        }
    }

    pub fn next(self) -> Self {
        match self {
            LogoQuality::Quality => LogoQuality::Medium,
            LogoQuality::Medium => LogoQuality::Pixel,
            LogoQuality::Pixel => LogoQuality::Quality,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            LogoQuality::Quality => LogoQuality::Pixel,
            LogoQuality::Medium => LogoQuality::Quality,
            LogoQuality::Pixel => LogoQuality::Medium,
        }
    }

    pub fn scale(self) -> u32 {
        match self {
            LogoQuality::Quality => 3,
            LogoQuality::Medium => 2,
            LogoQuality::Pixel => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Debug)]
pub struct LogoPalette {
    pub default: Option<RgbColor>,
    pub colors: Vec<RgbColor>,
}

impl Default for LogoPalette {
    fn default() -> Self {
        Self {
            default: Some(RgbColor {
                r: 255,
                g: 255,
                b: 255,
            }),
            colors: vec![
                RgbColor {
                    r: 78,
                    g: 190,
                    b: 210,
                },
                RgbColor {
                    r: 230,
                    g: 180,
                    b: 70,
                },
                RgbColor {
                    r: 230,
                    g: 90,
                    b: 70,
                },
                RgbColor {
                    r: 95,
                    g: 200,
                    b: 120,
                },
                RgbColor {
                    r: 138,
                    g: 148,
                    b: 158,
                },
                RgbColor {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                RgbColor {
                    r: 220,
                    g: 140,
                    b: 200,
                },
                RgbColor {
                    r: 120,
                    g: 180,
                    b: 255,
                },
                RgbColor {
                    r: 200,
                    g: 200,
                    b: 200,
                },
            ],
        }
    }
}

impl LogoPalette {
    pub fn color_for_index(&self, index: u8) -> Option<RgbColor> {
        if index == 0 {
            return self.default;
        }
        let idx = index.saturating_sub(1) as usize;
        self.colors.get(idx).copied().or(self.default)
    }
}

#[derive(Clone, Debug)]
pub struct AsciiCell {
    pub ch: char,
    pub color_index: Option<u8>,
}

impl AsciiCell {
    pub fn blank() -> Self {
        Self {
            ch: ' ',
            color_index: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsciiLogo {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<AsciiCell>>,
}

#[derive(Clone, Debug)]
pub struct SvgLogo {
    pub tree: usvg::Tree,
}

#[derive(Clone, Copy, Debug)]
pub struct LogoCell {
    pub ch: char,
    pub fg: Option<RgbColor>,
    pub bg: Option<RgbColor>,
}

impl LogoCell {
    pub fn blank() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderedLogo {
    pub mode: LogoMode,
    pub quality: LogoQuality,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<LogoCell>,
}

impl RenderedLogo {
    pub fn blank(mode: LogoMode, quality: LogoQuality, width: u16, height: u16) -> Self {
        let cells = vec![LogoCell::blank(); width as usize * height as usize];
        Self {
            mode,
            quality,
            width,
            height,
            cells,
        }
    }
}

#[derive(Debug, Default)]
pub struct LogoCache {
    pub ascii: Option<AsciiLogo>,
    pub svg: Option<SvgLogo>,
    pub palette: LogoPalette,
    pub rendered: Option<RenderedLogo>,
}
