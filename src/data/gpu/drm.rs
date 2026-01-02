use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::types::GpuProcessUsage;

type ProcessKey = (String, u32);

#[derive(Default)]
pub struct DrmProcessTracker {
    last: HashMap<ProcessKey, DrmProcessCounters>,
    last_instant: Option<Instant>,
}

impl DrmProcessTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample_processes(&mut self) -> Vec<GpuProcessUsage> {
        let now = Instant::now();
        let current = collect_drm_process_counters();
        let interval_ns = self
            .last_instant
            .map(|previous| now.saturating_duration_since(previous))
            .map(|elapsed| elapsed.as_secs_f64() * 1_000_000_000.0)
            .filter(|value| *value > 0.0);

        let mut outputs = Vec::new();
        for (key, counters) in &current {
            let (gpu_id, pid) = key;
            let mut usage = GpuProcessUsage {
                gpu_id: gpu_id.clone(),
                pid: *pid,
                kind: counters.kind,
                sm_pct: None,
                mem_pct: None,
                enc_pct: None,
                dec_pct: None,
                fb_mb: bytes_to_mb(counters.preferred_mem_bytes()),
            };

            if let Some(interval_ns) = interval_ns
                && let Some(prev) = self.last.get(key)
            {
                usage.sm_pct = compute_pct(counters.core_ns, prev.core_ns, interval_ns);
                usage.enc_pct = compute_pct(counters.enc_ns, prev.enc_ns, interval_ns);
                usage.dec_pct = compute_pct(counters.dec_ns, prev.dec_ns, interval_ns);
            }

            outputs.push(usage);
        }

        self.last = current;
        self.last_instant = Some(now);
        outputs
    }
}

#[derive(Default, Clone, Copy)]
struct DrmProcessCounters {
    core_ns: u64,
    enc_ns: u64,
    dec_ns: u64,
    vram_bytes: u64,
    system_bytes: u64,
    kind: Option<char>,
}

impl DrmProcessCounters {
    fn merge(&mut self, sample: DrmFdinfoSample) {
        self.core_ns = self.core_ns.saturating_add(sample.core_ns);
        self.enc_ns = self.enc_ns.saturating_add(sample.enc_ns);
        self.dec_ns = self.dec_ns.saturating_add(sample.dec_ns);
        self.vram_bytes = self.vram_bytes.saturating_add(sample.vram_bytes);
        self.system_bytes = self.system_bytes.saturating_add(sample.system_bytes);
        self.kind = merge_kind(self.kind, sample.kind);
    }

    fn preferred_mem_bytes(self) -> u64 {
        if self.vram_bytes > 0 {
            self.vram_bytes
        } else {
            self.system_bytes
        }
    }
}

#[derive(Default)]
struct DrmFdinfoSample {
    gpu_id: String,
    core_ns: u64,
    enc_ns: u64,
    dec_ns: u64,
    vram_bytes: u64,
    system_bytes: u64,
    kind: Option<char>,
}

fn collect_drm_process_counters() -> HashMap<ProcessKey, DrmProcessCounters> {
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return map;
    };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let fdinfo_path = entry.path().join("fdinfo");
        let Ok(fd_entries) = fs::read_dir(fdinfo_path) else {
            continue;
        };
        for fd_entry in fd_entries.flatten() {
            let Ok(contents) = fs::read_to_string(fd_entry.path()) else {
                continue;
            };
            let Some(sample) = parse_fdinfo(&contents) else {
                continue;
            };
            let key = (sample.gpu_id.clone(), pid);
            map.entry(key)
                .or_insert_with(DrmProcessCounters::default)
                .merge(sample);
        }
    }
    map
}

fn parse_fdinfo(contents: &str) -> Option<DrmFdinfoSample> {
    let mut driver = None;
    let mut pdev = None;
    let mut minor = None;
    let mut sample = DrmFdinfoSample::default();

    for line in contents.lines() {
        let (key, value) = line.split_once(':').map(|(k, v)| (k.trim(), v.trim()))?;
        match key {
            "drm-driver" => driver = Some(value),
            "drm-pdev" => pdev = value.split_whitespace().next(),
            "drm-minor" => {
                minor = value
                    .split_whitespace()
                    .next()
                    .and_then(|val| val.parse::<u32>().ok())
            }
            _ if key.starts_with("drm-engine-") => {
                let name = key.trim_start_matches("drm-engine-");
                if let Some(ns) = parse_engine_ns(value) {
                    match classify_engine(name) {
                        EngineClass::Core => sample.core_ns = sample.core_ns.saturating_add(ns),
                        EngineClass::Encode => sample.enc_ns = sample.enc_ns.saturating_add(ns),
                        EngineClass::Decode => sample.dec_ns = sample.dec_ns.saturating_add(ns),
                        EngineClass::Other => {}
                    }
                    sample.kind = merge_kind(sample.kind, engine_kind_hint(name));
                }
            }
            _ if key.starts_with("drm-memory-") => {
                if let Some(bytes) = parse_memory_bytes(value) {
                    match classify_memory(key) {
                        MemoryClass::Vram => {
                            sample.vram_bytes = sample.vram_bytes.saturating_add(bytes)
                        }
                        MemoryClass::System => {
                            sample.system_bytes = sample.system_bytes.saturating_add(bytes)
                        }
                        MemoryClass::Other => {}
                    }
                }
            }
            _ => {}
        }
    }

    driver?;
    let gpu_id = resolve_gpu_id(pdev, minor)?;
    sample.gpu_id = gpu_id;
    Some(sample)
}

fn resolve_gpu_id(pdev: Option<&str>, minor: Option<u32>) -> Option<String> {
    if let Some(pdev) = pdev
        && !pdev.is_empty()
    {
        return Some(format!("pci:{pdev}"));
    }
    let minor = minor?;
    let path = if minor >= 128 {
        Path::new("/sys/class/drm").join(format!("renderD{minor}"))
    } else {
        Path::new("/sys/class/drm").join(format!("card{minor}"))
    };
    let device = read_link_basename(path.join("device"))?;
    Some(format!("pci:{device}"))
}

fn read_link_basename(path: PathBuf) -> Option<String> {
    fs::read_link(path).ok().and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    })
}

fn parse_engine_ns(value: &str) -> Option<u64> {
    value.split_whitespace().next()?.parse::<u64>().ok()
}

fn parse_memory_bytes(value: &str) -> Option<u64> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<u64>().ok()?;
    let unit = parts.next().unwrap_or("B");
    let multiplier = match unit {
        "KiB" | "kB" => 1024,
        "MiB" => 1024 * 1024,
        "GiB" => 1024 * 1024 * 1024,
        _ => 1,
    };
    Some(amount.saturating_mul(multiplier))
}

fn bytes_to_mb(bytes: u64) -> Option<u64> {
    if bytes == 0 {
        None
    } else {
        Some(bytes.div_ceil(1024 * 1024))
    }
}

fn compute_pct(current: u64, previous: u64, interval_ns: f64) -> Option<f32> {
    if interval_ns <= 0.0 {
        return None;
    }
    let delta = current.saturating_sub(previous) as f64;
    let pct = ((delta / interval_ns) * 100.0).clamp(0.0, 100.0);
    Some(pct as f32)
}

fn merge_kind(current: Option<char>, incoming: Option<char>) -> Option<char> {
    match (current, incoming) {
        (Some('C'), _) => Some('C'),
        (Some('G'), Some('C')) => Some('C'),
        (None, Some(kind)) => Some(kind),
        (Some(kind), _) => Some(kind),
        _ => None,
    }
}

fn engine_kind_hint(name: &str) -> Option<char> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("compute") {
        Some('C')
    } else if lower.contains("render") || lower.contains("gfx") || lower.contains("3d") {
        Some('G')
    } else {
        None
    }
}

enum EngineClass {
    Core,
    Encode,
    Decode,
    Other,
}

fn classify_engine(name: &str) -> EngineClass {
    let lower = name.to_ascii_lowercase();
    if lower.contains("enc") || lower.contains("encode") {
        return EngineClass::Encode;
    }
    if lower.contains("dec") || lower.contains("decode") || lower.contains("video") {
        return EngineClass::Decode;
    }
    if lower.contains("render")
        || lower.contains("gfx")
        || lower.contains("compute")
        || lower.contains("3d")
    {
        return EngineClass::Core;
    }
    EngineClass::Other
}

enum MemoryClass {
    Vram,
    System,
    Other,
}

fn classify_memory(key: &str) -> MemoryClass {
    if key.contains("vram") || key.contains("local") {
        MemoryClass::Vram
    } else if key.contains("system") || key.contains("gtt") || key.contains("shared") {
        MemoryClass::System
    } else {
        MemoryClass::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fdinfo_sample_collects_fields() {
        let content = concat!(
            "pos:\t0\n",
            "flags:\t02000002\n",
            "mnt_id:\t12\n",
            "ino:\t12345\n",
            "drm-driver:\tamdgpu\n",
            "drm-pdev:\t0000:03:00.0\n",
            "drm-engine-gfx:\t1000 ns\n",
            "drm-engine-encode:\t500 ns\n",
            "drm-memory-vram:\t1024 KiB\n",
            "drm-memory-gtt:\t2048 KiB\n",
        );

        let sample = parse_fdinfo(content).unwrap();
        assert_eq!(sample.gpu_id, "pci:0000:03:00.0");
        assert_eq!(sample.core_ns, 1000);
        assert_eq!(sample.enc_ns, 500);
        assert_eq!(sample.dec_ns, 0);
        assert_eq!(sample.vram_bytes, 1024 * 1024);
        assert_eq!(sample.system_bytes, 2048 * 1024);
        assert_eq!(sample.kind, Some('G'));
    }
}
