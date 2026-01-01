#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Overview,
    Processes,
    GpuFocus,
    SystemInfo,
    Container,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Overview => "Overview",
            ViewMode::Processes => "Processes",
            ViewMode::GpuFocus => "GPU",
            ViewMode::SystemInfo => "System",
            ViewMode::Container => "Containers",
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Overview
    }
}
