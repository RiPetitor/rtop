mod lspci;
mod monitor;
mod nvidia;
mod provider;
mod sysfs;
mod types;

pub use monitor::start_gpu_monitor;
pub use provider::{
    GpuProvider, GpuProviderRegistry, LspciProvider, NvidiaProvider, SysfsProvider,
};
pub use types::{GpuInfo, GpuKind, GpuMemory, GpuPreference, GpuSnapshot, PciName};

use std::collections::HashMap;

use crate::utils::text_width;

pub fn probe_gpus() -> GpuSnapshot {
    let pci_names = pci_name_map();
    let registry = GpuProviderRegistry::with_defaults();
    let mut gpus = registry.probe_all();
    normalize_gpu_names(&mut gpus, &pci_names);
    GpuSnapshot { gpus }
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

fn merge_gpu_info(current: &mut GpuInfo, incoming: &GpuInfo) {
    if current.kind == GpuKind::Unknown && incoming.kind != GpuKind::Unknown {
        current.kind = incoming.kind;
    }
    if current.memory.is_none() && incoming.memory.is_some() {
        current.memory.clone_from(&incoming.memory);
    }
    if current.vendor.is_none() && incoming.vendor.is_some() {
        current.vendor.clone_from(&incoming.vendor);
    }
    if current.device.is_none() && incoming.device.is_some() {
        current.device.clone_from(&incoming.device);
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
                kind: GpuKind::Integrated,
                memory: None,
            },
            GpuInfo {
                id: "nvidia:0".to_string(),
                name: "RTX".to_string(),
                vendor: Some("NVIDIA".to_string()),
                device: Some("RTX".to_string()),
                kind: GpuKind::Discrete,
                memory: None,
            },
        ];

        assert_eq!(default_gpu_index(&gpus, GpuPreference::Auto), Some(1));
        assert_eq!(default_gpu_index(&gpus, GpuPreference::Integrated), Some(0));
    }
}
