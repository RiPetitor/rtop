use crate::app::IconMode;

pub struct IconLabel {
    nerd: &'static str,
    text: &'static str,
}

impl IconLabel {
    pub const fn new(nerd: &'static str, text: &'static str) -> Self {
        Self { nerd, text }
    }

    pub fn get(&self, icon_mode: IconMode) -> &'static str {
        match icon_mode {
            IconMode::Nerd => self.nerd,
            IconMode::Text => self.text,
        }
    }
}

pub const ICON_USER: IconLabel = IconLabel::new("", "User:");
pub const ICON_DISTRO: IconLabel = IconLabel::new("󱋩", "Distro:");
pub const ICON_OS: IconLabel = IconLabel::new("󰣛", "OS:");
pub const ICON_KERNEL: IconLabel = IconLabel::new("", "Kernel:");
pub const ICON_UPTIME: IconLabel = IconLabel::new("󰅐", "Uptime:");
pub const ICON_BOARD: IconLabel = IconLabel::new("󰾰", "Board:");
pub const ICON_CPU: IconLabel = IconLabel::new("󰻠", "CPU:");
pub const ICON_GPU: IconLabel = IconLabel::new("󰍛", "GPU:");
pub const ICON_MEM: IconLabel = IconLabel::new("", "RAM:");
pub const ICON_DISK: IconLabel = IconLabel::new("", "Disk:");
pub const ICON_DISPLAY: IconLabel = IconLabel::new("󰍹", "Display:");
pub const ICON_MOUSE: IconLabel = IconLabel::new("󰖺", "Mouse:");
pub const ICON_DE: IconLabel = IconLabel::new("󰕮", "DE:");
pub const ICON_WM: IconLabel = IconLabel::new("", "WM:");
pub const ICON_SHELL: IconLabel = IconLabel::new("", "Shell:");
pub const ICON_TERM: IconLabel = IconLabel::new("", "Term:");
pub const ICON_PKG: IconLabel = IconLabel::new("󰏖", "Pkgs:");
pub const ICON_SEP_NERD: &str = "";
pub const ICON_IMMUTABLE: &str = "";
