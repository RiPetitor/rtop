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

#[derive(Clone, Debug, Default)]
pub struct GpuTelemetry {
    pub utilization_gpu_pct: Option<f32>,
    pub utilization_mem_pct: Option<f32>,
    pub temperature_c: Option<f32>,
    pub power_draw_w: Option<f32>,
    pub power_limit_w: Option<f32>,
    pub fan_speed_pct: Option<f32>,
    pub encoder_pct: Option<f32>,
    pub decoder_pct: Option<f32>,
}

impl GpuTelemetry {
    pub fn merge_from(&mut self, other: &Self) {
        self.utilization_gpu_pct = self.utilization_gpu_pct.or(other.utilization_gpu_pct);
        self.utilization_mem_pct = self.utilization_mem_pct.or(other.utilization_mem_pct);
        self.temperature_c = self.temperature_c.or(other.temperature_c);
        self.power_draw_w = self.power_draw_w.or(other.power_draw_w);
        self.power_limit_w = self.power_limit_w.or(other.power_limit_w);
        self.fan_speed_pct = self.fan_speed_pct.or(other.fan_speed_pct);
        self.encoder_pct = self.encoder_pct.or(other.encoder_pct);
        self.decoder_pct = self.decoder_pct.or(other.decoder_pct);
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
    pub telemetry: GpuTelemetry,
}

#[derive(Clone, Debug)]
pub struct GpuProcessUsage {
    pub gpu_id: String,
    pub pid: u32,
    pub kind: Option<char>,
    pub sm_pct: Option<f32>,
    pub mem_pct: Option<f32>,
    pub enc_pct: Option<f32>,
    pub dec_pct: Option<f32>,
    pub fb_mb: Option<u64>,
}

#[derive(Debug)]
pub struct GpuSnapshot {
    pub gpus: Vec<GpuInfo>,
    pub processes: Vec<GpuProcessUsage>,
}

#[derive(Clone)]
pub struct PciName {
    pub vendor: String,
    pub device: String,
}
