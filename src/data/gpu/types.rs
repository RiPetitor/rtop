#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpuKind {
    Discrete,
    Integrated,
    Unknown,
}

impl GpuKind {
    pub fn sort_rank(self) -> u8 {
        match self {
            GpuKind::Discrete => 0,
            GpuKind::Integrated => 1,
            GpuKind::Unknown => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpuPreference {
    Auto,
    Discrete,
    Integrated,
}

impl GpuPreference {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Some(GpuPreference::Auto),
            "discrete" | "dgpu" => Some(GpuPreference::Discrete),
            "integrated" | "igpu" => Some(GpuPreference::Integrated),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GpuMemory {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub id: String,
    pub name: String,
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub kind: GpuKind,
    pub memory: Option<GpuMemory>,
}

#[derive(Debug)]
pub struct GpuSnapshot {
    pub gpus: Vec<GpuInfo>,
}

#[derive(Clone)]
pub struct PciName {
    pub vendor: String,
    pub device: String,
}
