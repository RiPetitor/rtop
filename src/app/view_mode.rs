#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
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
