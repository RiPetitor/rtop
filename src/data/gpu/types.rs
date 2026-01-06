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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    pub driver: Option<String>,
    pub driver_version: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_kind_sort_rank() {
        assert_eq!(GpuKind::Discrete.sort_rank(), 0);
        assert_eq!(GpuKind::Integrated.sort_rank(), 1);
        assert_eq!(GpuKind::Unknown.sort_rank(), 2);
    }

    #[test]
    fn gpu_kind_sort_order() {
        assert!(GpuKind::Discrete.sort_rank() < GpuKind::Integrated.sort_rank());
        assert!(GpuKind::Integrated.sort_rank() < GpuKind::Unknown.sort_rank());
    }

    #[test]
    fn gpu_preference_parse_auto() {
        assert_eq!(GpuPreference::parse("auto"), Some(GpuPreference::Auto));
        assert_eq!(GpuPreference::parse("AUTO"), Some(GpuPreference::Auto));
        assert_eq!(GpuPreference::parse("Auto"), Some(GpuPreference::Auto));
    }

    #[test]
    fn gpu_preference_parse_discrete() {
        assert_eq!(
            GpuPreference::parse("discrete"),
            Some(GpuPreference::Discrete)
        );
        assert_eq!(GpuPreference::parse("dgpu"), Some(GpuPreference::Discrete));
        assert_eq!(
            GpuPreference::parse("DISCRETE"),
            Some(GpuPreference::Discrete)
        );
    }

    #[test]
    fn gpu_preference_parse_integrated() {
        assert_eq!(
            GpuPreference::parse("integrated"),
            Some(GpuPreference::Integrated)
        );
        assert_eq!(
            GpuPreference::parse("igpu"),
            Some(GpuPreference::Integrated)
        );
        assert_eq!(
            GpuPreference::parse("INTEGRATED"),
            Some(GpuPreference::Integrated)
        );
    }

    #[test]
    fn gpu_preference_parse_invalid() {
        assert_eq!(GpuPreference::parse("invalid"), None);
        assert_eq!(GpuPreference::parse(""), None);
        assert_eq!(GpuPreference::parse("  "), None);
    }

    #[test]
    fn gpu_telemetry_merge_from() {
        let mut telemetry = GpuTelemetry {
            utilization_gpu_pct: Some(50.0),
            utilization_mem_pct: None,
            temperature_c: None,
            power_draw_w: None,
            power_limit_w: None,
            fan_speed_pct: None,
            encoder_pct: None,
            decoder_pct: None,
        };

        let other = GpuTelemetry {
            utilization_gpu_pct: Some(60.0),
            utilization_mem_pct: Some(70.0),
            temperature_c: Some(45.0),
            power_draw_w: Some(10.5),
            power_limit_w: Some(200.0),
            fan_speed_pct: Some(30.0),
            encoder_pct: Some(20.0),
            decoder_pct: Some(15.0),
        };

        telemetry.merge_from(&other);

        assert_eq!(telemetry.utilization_gpu_pct, Some(50.0));
        assert_eq!(telemetry.utilization_mem_pct, Some(70.0));
        assert_eq!(telemetry.temperature_c, Some(45.0));
        assert_eq!(telemetry.power_draw_w, Some(10.5));
        assert_eq!(telemetry.power_limit_w, Some(200.0));
        assert_eq!(telemetry.fan_speed_pct, Some(30.0));
        assert_eq!(telemetry.encoder_pct, Some(20.0));
        assert_eq!(telemetry.decoder_pct, Some(15.0));
    }

    #[test]
    fn gpu_telemetry_merge_keeps_existing() {
        let mut telemetry = GpuTelemetry {
            utilization_gpu_pct: Some(50.0),
            ..Default::default()
        };

        let other = GpuTelemetry {
            utilization_gpu_pct: Some(60.0),
            ..Default::default()
        };

        telemetry.merge_from(&other);

        assert_eq!(telemetry.utilization_gpu_pct, Some(50.0));
    }

    #[test]
    fn gpu_telemetry_default() {
        let telemetry = GpuTelemetry::default();
        assert!(telemetry.utilization_gpu_pct.is_none());
        assert!(telemetry.utilization_mem_pct.is_none());
        assert!(telemetry.temperature_c.is_none());
        assert!(telemetry.power_draw_w.is_none());
        assert!(telemetry.power_limit_w.is_none());
        assert!(telemetry.fan_speed_pct.is_none());
        assert!(telemetry.encoder_pct.is_none());
        assert!(telemetry.decoder_pct.is_none());
    }

    #[test]
    fn gpu_memory_calculates_percent() {
        let memory = GpuMemory {
            used_bytes: 1024 * 1024 * 1024,
            total_bytes: 2 * 1024 * 1024 * 1024,
        };
        let percent = (memory.used_bytes as f32 / memory.total_bytes as f32) * 100.0;
        assert_eq!(percent, 50.0);
    }

    #[test]
    fn gpu_info_default_telemetry() {
        let gpu = GpuInfo {
            id: "test".to_string(),
            name: "Test GPU".to_string(),
            vendor: None,
            device: None,
            driver: None,
            driver_version: None,
            kind: GpuKind::Discrete,
            memory: None,
            telemetry: Default::default(),
        };

        assert!(gpu.telemetry.utilization_gpu_pct.is_none());
        assert!(gpu.memory.is_none());
    }

    #[test]
    fn gpu_process_usage_defaults() {
        let usage = GpuProcessUsage {
            gpu_id: "gpu:0".to_string(),
            pid: 1234,
            kind: None,
            sm_pct: None,
            mem_pct: None,
            enc_pct: None,
            dec_pct: None,
            fb_mb: None,
        };

        assert!(usage.kind.is_none());
        assert!(usage.sm_pct.is_none());
        assert!(usage.mem_pct.is_none());
        assert!(usage.enc_pct.is_none());
        assert!(usage.dec_pct.is_none());
        assert!(usage.fb_mb.is_none());
    }

    #[test]
    fn pci_name_fields() {
        let pci_name = PciName {
            vendor: "NVIDIA".to_string(),
            device: "GeForce RTX 3080".to_string(),
        };

        assert_eq!(pci_name.vendor, "NVIDIA");
        assert_eq!(pci_name.device, "GeForce RTX 3080");
    }
}
