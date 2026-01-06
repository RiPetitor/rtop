mod drm;
mod lspci;
mod monitor;
mod nvidia;
mod provider;
mod sysfs;
mod types;

pub use drm::DrmProcessTracker;
pub use monitor::start_gpu_monitor;
pub use provider::{
    GpuProvider, GpuProviderRegistry, LspciProvider, NvidiaProvider, SysfsProvider,
};
pub use types::{
    GpuInfo, GpuKind, GpuMemory, GpuPreference, GpuProcessUsage, GpuSnapshot, GpuTelemetry, PciName,
};

use std::collections::HashMap;
use std::time::Duration;

use crate::utils::text_width;

pub fn probe_gpus() -> GpuSnapshot {
    let mut tracker = DrmProcessTracker::new();
    probe_gpus_with_tracker(&mut tracker)
}

pub fn probe_gpus_with_tracker(tracker: &mut DrmProcessTracker) -> GpuSnapshot {
    let pci_names = pci_name_map();
    let registry = GpuProviderRegistry::with_defaults();
    let mut gpus = registry.probe_all();
    normalize_gpu_names(&mut gpus, &pci_names);
    let mut process_sources = Vec::new();
    let has_nvidia = gpus.iter().any(|gpu| gpu.id.starts_with("nvidia:"));
    let needs_drm = gpus.iter().any(|gpu| !gpu.id.starts_with("nvidia:"));
    if has_nvidia {
        process_sources.push(nvidia::probe_nvidia_processes(Duration::from_millis(800)));
    }
    if needs_drm {
        process_sources.push(tracker.sample_processes());
    }
    let processes = merge_process_lists(process_sources);
    GpuSnapshot { gpus, processes }
}

#[cfg(all(target_os = "linux", feature = "pci-names"))]
fn pci_name_map() -> HashMap<String, PciName> {
    use libpci::{Fill, PCIAccess};

    let mut map = HashMap::new();
    let Some(mut access) = PCIAccess::try_new(true) else {
        return map;
    };
    let Some(device) = access.devices() else {
        return map;
    };
    for mut dev in device.iter_mut() {
        dev.fill_info(Fill::IDENT as u32);
        let (Some(domain), Some(bus), Some(dev_id), Some(func)) =
            (dev.domain(), dev.bus(), dev.dev(), dev.func())
        else {
            continue;
        };
        let vendor = dev.vendor().unwrap_or_default();
        let device = dev.device().unwrap_or_default();
        if vendor.is_empty() && device.is_empty() {
            continue;
        }
        let slot = format!(
            "{:04x}:{:02x}:{:02x}.{:x}",
            domain as u16, bus, dev_id, func
        );
        map.insert(slot, PciName { vendor, device });
    }
    map
}

#[cfg(not(all(target_os = "linux", feature = "pci-names")))]
fn pci_name_map() -> HashMap<String, PciName> {
    HashMap::new()
}

pub fn merge_gpu_lists_multi(sources: Vec<Vec<GpuInfo>>) -> Vec<GpuInfo> {
    let mut by_id: HashMap<String, GpuInfo> = HashMap::new();
    for list in sources {
        for gpu in list {
            by_id
                .entry(gpu.id.clone())
                .and_modify(|current| merge_gpu_info(current, &gpu))
                .or_insert(gpu);
        }
    }
    by_id.into_values().collect()
}

fn merge_process_lists(sources: Vec<Vec<GpuProcessUsage>>) -> Vec<GpuProcessUsage> {
    let mut by_key: HashMap<(String, u32), GpuProcessUsage> = HashMap::new();
    for list in sources {
        for usage in list {
            by_key
                .entry((usage.gpu_id.clone(), usage.pid))
                .and_modify(|current| merge_process_usage(current, &usage))
                .or_insert(usage);
        }
    }
    by_key.into_values().collect()
}

fn merge_process_usage(current: &mut GpuProcessUsage, incoming: &GpuProcessUsage) {
    current.kind = merge_kind(current.kind, incoming.kind);
    merge_optional_max(&mut current.sm_pct, incoming.sm_pct);
    merge_optional_max(&mut current.mem_pct, incoming.mem_pct);
    merge_optional_max(&mut current.enc_pct, incoming.enc_pct);
    merge_optional_max(&mut current.dec_pct, incoming.dec_pct);
    if let Some(fb_mb) = incoming.fb_mb {
        let merged = current.fb_mb.unwrap_or(0).max(fb_mb);
        current.fb_mb = Some(merged);
    }
}

fn merge_optional_max(current: &mut Option<f32>, incoming: Option<f32>) {
    let Some(value) = incoming else {
        return;
    };
    match current {
        Some(existing) => {
            if value > *existing {
                *current = Some(value);
            }
        }
        None => {
            *current = Some(value);
        }
    }
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

fn merge_gpu_info(current: &mut GpuInfo, incoming: &GpuInfo) {
    if current.kind == GpuKind::Unknown && incoming.kind != GpuKind::Unknown {
        current.kind = incoming.kind;
    }
    if current.memory.is_none() && incoming.memory.is_some() {
        current.memory.clone_from(&incoming.memory);
    }
    current.telemetry.merge_from(&incoming.telemetry);
    if current.vendor.is_none() && incoming.vendor.is_some() {
        current.vendor.clone_from(&incoming.vendor);
    }
    if current.device.is_none() && incoming.device.is_some() {
        current.device.clone_from(&incoming.device);
    }
    if current.driver.is_none() && incoming.driver.is_some() {
        current.driver.clone_from(&incoming.driver);
    }
    if current.driver_version.is_none() && incoming.driver_version.is_some() {
        current.driver_version.clone_from(&incoming.driver_version);
    }
    if text_width(&incoming.name) > text_width(&current.name) {
        current.name.clone_from(&incoming.name);
    }
}

fn normalize_gpu_names(gpus: &mut [GpuInfo], names: &HashMap<String, PciName>) {
    for gpu in gpus {
        let Some(slot) = gpu.id.strip_prefix("pci:") else {
            continue;
        };
        let Some(info) = names.get(slot) else {
            continue;
        };
        gpu.vendor = Some(info.vendor.clone());
        gpu.device = Some(info.device.clone());
        let name = format!("{} {}", info.vendor, info.device)
            .trim()
            .to_string();
        if !name.is_empty() {
            gpu.name = name;
        }
    }
}

pub fn default_gpu_index(gpus: &[GpuInfo], pref: GpuPreference) -> Option<usize> {
    let order = match pref {
        GpuPreference::Auto | GpuPreference::Discrete => {
            [GpuKind::Discrete, GpuKind::Integrated, GpuKind::Unknown]
        }
        GpuPreference::Integrated => [GpuKind::Integrated, GpuKind::Discrete, GpuKind::Unknown],
    };
    for kind in order {
        if let Some(idx) = gpus.iter().position(|gpu| gpu.kind == kind) {
            return Some(idx);
        }
    }
    None
}

pub fn gpu_vendor_label(gpu: &GpuInfo) -> String {
    if let Some(vendor) = gpu.vendor.as_deref() {
        return short_vendor_name(vendor).to_string();
    }
    if gpu.id.starts_with("nvidia:") {
        return "NVIDIA".to_string();
    }
    let name = gpu.name.to_ascii_lowercase();
    if name.contains("nvidia")
        || name.contains("geforce")
        || name.contains("quadro")
        || name.contains("tesla")
    {
        return "NVIDIA".to_string();
    }
    if name.contains("amd")
        || name.contains("ati")
        || name.contains("radeon")
        || name.contains("advanced micro devices")
    {
        return "AMD".to_string();
    }
    if name.contains("intel")
        || name.contains("iris")
        || name.contains("uhd")
        || name.contains("arc")
    {
        return "Intel".to_string();
    }
    "GPU".to_string()
}

fn short_vendor_name(vendor: &str) -> &'static str {
    let lower = vendor.to_ascii_lowercase();
    if lower.contains("nvidia") {
        return "NVIDIA";
    }
    if lower.contains("amd") || lower.contains("ati") || lower.contains("advanced micro devices") {
        return "AMD";
    }
    if lower.contains("intel") {
        return "Intel";
    }
    "GPU"
}

/// Extracts short GPU model name from full description
/// "Navi 32 [Radeon RX 7700 XT / 7800 XT]" -> "RX 7700 XT"
/// "Advanced Micro Devices, Inc. [AMD/ATI] Navi 32 [Radeon RX 7700 XT / 7800 XT]" -> "RX 7700 XT"
/// "GeForce RTX 4080" -> "RTX 4080"
pub fn short_device_name(device: &str) -> String {
    // Find last square brackets (model is usually there)
    // Skip [AMD/ATI], [NVIDIA] and similar vendor tags
    let mut best_match: Option<String> = None;
    let mut search_from = 0;

    while let Some(start) = device[search_from..].find('[') {
        let abs_start = search_from + start;
        if let Some(end) = device[abs_start..].find(']') {
            let bracket_content = &device[abs_start + 1..abs_start + end];

            // Skip short vendor tags like [AMD/ATI], [NVIDIA]
            if !bracket_content.contains("AMD/ATI")
                && !bracket_content.contains("NVIDIA")
                && !bracket_content.contains("Intel")
                && bracket_content.len() > 5
            {
                // Extract first model before " / "
                let model = bracket_content
                    .split(" / ")
                    .next()
                    .unwrap_or(bracket_content);
                // Remove prefixes like "Radeon "
                let cleaned = model
                    .trim_start_matches("Radeon ")
                    .trim_start_matches("GeForce ")
                    .trim_start_matches("Intel ")
                    .trim_start_matches("Arc ");
                if !cleaned.is_empty() {
                    best_match = Some(cleaned.to_string());
                }
            }
            search_from = abs_start + end + 1;
        } else {
            break;
        }
    }

    if let Some(name) = best_match {
        return name;
    }

    // Search for known model patterns
    let patterns = ["RTX ", "GTX ", "RX ", "Arc ", "Iris ", "UHD ", "Quadro "];
    for pattern in patterns {
        if let Some(pos) = device.find(pattern) {
            let rest = &device[pos..];
            // Take until end of model word/number
            let end = rest
                .find(|c| ['[', '(', ','].contains(&c))
                .unwrap_or(rest.len());
            return rest[..end].trim().to_string();
        }
    }

    // Fallback: first 20 characters
    if device.len() > 20 {
        format!("{}...", &device[..20])
    } else {
        device.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gpu_index_prefers_discrete() {
        let gpus = vec![
            GpuInfo {
                id: "pci:00:02.0".to_string(),
                name: "Intel UHD".to_string(),
                vendor: Some("Intel".to_string()),
                device: Some("UHD".to_string()),
                driver: None,
                driver_version: None,
                kind: GpuKind::Integrated,
                memory: None,
                telemetry: GpuTelemetry::default(),
            },
            GpuInfo {
                id: "nvidia:0".to_string(),
                name: "RTX".to_string(),
                vendor: Some("NVIDIA".to_string()),
                device: Some("RTX".to_string()),
                driver: None,
                driver_version: None,
                kind: GpuKind::Discrete,
                memory: None,
                telemetry: GpuTelemetry::default(),
            },
        ];

        assert_eq!(default_gpu_index(&gpus, GpuPreference::Auto), Some(1));
        assert_eq!(default_gpu_index(&gpus, GpuPreference::Integrated), Some(0));
    }
}
