use ratatui::prelude::Rect;

use crate::data::{SortDir, SortKey};

pub struct ConfirmKill {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub start_time: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Russian",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" => Some(Language::English),
            "ru" | "rus" | "russian" => Some(Language::Russian),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Language::English => Language::Russian,
            Language::Russian => Language::English,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GpuProcessSortKey {
    #[default]
    Pid,
    Kind,
    Sm,
    Mem,
    Enc,
    Dec,
    Vram,
    Name,
}

impl GpuProcessSortKey {
    pub fn default_dir(self) -> SortDir {
        match self {
            GpuProcessSortKey::Pid | GpuProcessSortKey::Kind | GpuProcessSortKey::Name => {
                SortDir::Asc
            }
            GpuProcessSortKey::Sm
            | GpuProcessSortKey::Mem
            | GpuProcessSortKey::Enc
            | GpuProcessSortKey::Dec
            | GpuProcessSortKey::Vram => SortDir::Desc,
        }
    }

    pub fn next(self) -> Self {
        match self {
            GpuProcessSortKey::Pid => GpuProcessSortKey::Kind,
            GpuProcessSortKey::Kind => GpuProcessSortKey::Sm,
            GpuProcessSortKey::Sm => GpuProcessSortKey::Mem,
            GpuProcessSortKey::Mem => GpuProcessSortKey::Enc,
            GpuProcessSortKey::Enc => GpuProcessSortKey::Dec,
            GpuProcessSortKey::Dec => GpuProcessSortKey::Vram,
            GpuProcessSortKey::Vram => GpuProcessSortKey::Name,
            GpuProcessSortKey::Name => GpuProcessSortKey::Pid,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            GpuProcessSortKey::Pid => GpuProcessSortKey::Name,
            GpuProcessSortKey::Kind => GpuProcessSortKey::Pid,
            GpuProcessSortKey::Sm => GpuProcessSortKey::Kind,
            GpuProcessSortKey::Mem => GpuProcessSortKey::Sm,
            GpuProcessSortKey::Enc => GpuProcessSortKey::Mem,
            GpuProcessSortKey::Dec => GpuProcessSortKey::Enc,
            GpuProcessSortKey::Vram => GpuProcessSortKey::Dec,
            GpuProcessSortKey::Name => GpuProcessSortKey::Vram,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemTab {
    #[default]
    Overview,
    Cpu,
    Memory,
    Disks,
    Network,
    Temps,
}

impl SystemTab {
    pub fn next(self) -> Self {
        match self {
            SystemTab::Overview => SystemTab::Cpu,
            SystemTab::Cpu => SystemTab::Memory,
            SystemTab::Memory => SystemTab::Disks,
            SystemTab::Disks => SystemTab::Network,
            SystemTab::Network => SystemTab::Temps,
            SystemTab::Temps => SystemTab::Overview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SystemTab::Overview => SystemTab::Temps,
            SystemTab::Cpu => SystemTab::Overview,
            SystemTab::Memory => SystemTab::Cpu,
            SystemTab::Disks => SystemTab::Memory,
            SystemTab::Network => SystemTab::Disks,
            SystemTab::Temps => SystemTab::Network,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SetupField {
    #[default]
    Language,
    IconMode,
    LogoMode,
    LogoQuality,
}

impl SetupField {
    pub fn next(self) -> Self {
        match self {
            SetupField::Language => SetupField::IconMode,
            SetupField::IconMode => SetupField::LogoMode,
            SetupField::LogoMode => SetupField::LogoQuality,
            SetupField::LogoQuality => SetupField::Language,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SetupField::Language => SetupField::LogoQuality,
            SetupField::IconMode => SetupField::Language,
            SetupField::LogoMode => SetupField::IconMode,
            SetupField::LogoQuality => SetupField::LogoMode,
        }
    }
}

#[derive(Clone, Copy)]
pub struct HeaderRegion {
    pub key: SortKey,
    pub rect: Rect,
}

#[derive(Clone, Copy)]
pub struct GpuProcessHeaderRegion {
    pub key: GpuProcessSortKey,
    pub rect: Rect,
}

#[derive(Clone, Copy)]
pub struct SystemTabRegion {
    pub tab: SystemTab,
    pub rect: Rect,
}

#[derive(Clone, Debug)]
pub struct SystemOverviewSnapshot {
    pub user_host: String,
    pub distro_line: String,
    pub os_name: String,
    pub kernel_line: String,
    pub uptime_line: String,
    pub board_line: String,
    pub cpu_line: String,
    pub gpu_line: String,
    pub mem_line: String,
    pub disk_lines: Vec<String>,
    pub display_line: String,
    pub mouse_line: String,
    pub de_line: String,
    pub wm_line: String,
    pub shell_line: String,
    pub terminal_line: String,
    pub package_line: String,
}
