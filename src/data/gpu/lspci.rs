use std::time::Duration;

use super::types::{GpuInfo, GpuKind};
use crate::utils::run_command_with_timeout;

pub fn probe_lspci_gpus(timeout: Duration, skip_nvidia: bool) -> Vec<GpuInfo> {
    let output = run_command_with_timeout("lspci", &["-mm", "-D"], timeout);
    if let Some(data) = output.as_deref() {
        let gpus = parse_lspci_output(data, skip_nvidia);
        if !gpus.is_empty() {
            return gpus;
        }
    }

    let output = run_command_with_timeout("lspci", &["-D"], timeout);
    output
        .as_deref()
        .map(|data| parse_lspci_legacy_output(data, skip_nvidia))
        .unwrap_or_default()
}

fn parse_lspci_output(output: &str, skip_nvidia: bool) -> Vec<GpuInfo> {
    output
        .lines()
        .filter_map(|line| {
            let entry = LspciEntry::parse(line)?;
            if !entry.is_display_controller() {
                return None;
            }
            if skip_nvidia && entry.vendor.to_ascii_lowercase().contains("nvidia") {
                return None;
            }
            let name = format!("{} {}", entry.vendor, entry.device)
                .trim()
                .to_string();
            Some(GpuInfo {
                id: format!("pci:{}", entry.slot),
                name,
                vendor: Some(entry.vendor.clone()),
                device: Some(entry.device.clone()),
                kind: classify_gpu_kind(&entry),
                memory: None,
            })
        })
        .collect()
}

fn parse_lspci_legacy_output(output: &str, skip_nvidia: bool) -> Vec<GpuInfo> {
    output
        .lines()
        .filter_map(|line| {
            let (slot, rest) = line.split_once(' ')?;
            let (class, desc) = rest.split_once(':')?;
            let class_lower = class.trim().to_ascii_lowercase();
            if !(class_lower.contains("vga")
                || class_lower.contains("3d controller")
                || class_lower.contains("display controller"))
            {
                return None;
            }
            let desc = desc.trim();
            if skip_nvidia && desc.to_ascii_lowercase().contains("nvidia") {
                return None;
            }
            let vendor = guess_vendor_name(desc);
            Some(GpuInfo {
                id: format!("pci:{slot}"),
                name: desc.to_string(),
                vendor: Some(vendor.clone()),
                device: Some(desc.to_string()),
                kind: classify_gpu_kind_fields(&vendor, desc, Some(slot), None),
                memory: None,
            })
        })
        .collect()
}

struct LspciEntry {
    slot: String,
    class: String,
    vendor: String,
    device: String,
}

impl LspciEntry {
    fn parse(line: &str) -> Option<Self> {
        let mut slot = None;
        let mut class = None;
        let mut vendor = None;
        let mut device = None;

        let mut parts = line.split('"');
        while let Some(key_part) = parts.next() {
            let Some(value) = parts.next() else {
                break;
            };
            let key = key_part.trim().trim_end_matches(':');
            match key {
                "Slot" => slot = Some(value.to_string()),
                "Class" => class = Some(value.to_string()),
                "Vendor" => vendor = Some(value.to_string()),
                "Device" => device = Some(value.to_string()),
                _ => {}
            }
        }

        Some(Self {
            slot: slot?,
            class: class?,
            vendor: vendor?,
            device: device?,
        })
    }

    fn is_display_controller(&self) -> bool {
        let class = self.class.to_ascii_lowercase();
        class.contains("vga")
            || class.contains("3d controller")
            || class.contains("display controller")
    }
}

fn classify_gpu_kind(entry: &LspciEntry) -> GpuKind {
    classify_gpu_kind_fields(
        &entry.vendor,
        &entry.device,
        Some(entry.slot.as_str()),
        None,
    )
}

pub fn classify_gpu_kind_fields(
    vendor: &str,
    device: &str,
    slot: Option<&str>,
    vendor_id: Option<u32>,
) -> GpuKind {
    let vendor_lower = vendor.to_ascii_lowercase();
    let device_lower = device.to_ascii_lowercase();
    let is_intel = vendor_id == Some(0x8086) || vendor_lower.contains("intel");
    let is_nvidia = vendor_id == Some(0x10de) || vendor_lower.contains("nvidia");
    let is_amd = vendor_id == Some(0x1002)
        || vendor_id == Some(0x1022)
        || vendor_lower.contains("amd")
        || vendor_lower.contains("ati");

    if is_intel {
        return GpuKind::Integrated;
    }
    if is_nvidia {
        return GpuKind::Discrete;
    }

    let slot_integrated = slot.is_some_and(is_integrated_slot);
    let integrated_name = device_lower.contains("integrated")
        || device_lower.contains("apu")
        || device_lower.contains("radeon graphics")
        || device_lower.contains("uhd")
        || device_lower.contains("iris")
        || device_lower.contains("vega 8")
        || device_lower.contains("vega 7")
        || device_lower.contains("vega 6");

    if is_amd {
        let amd_discrete = device_lower.contains("rx")
            || device_lower.contains("radeon pro")
            || device_lower.contains("firepro")
            || device_lower.contains("instinct")
            || device_lower.contains("radeon vii")
            || device_lower.contains("radeon vega");

        if integrated_name || slot_integrated {
            return GpuKind::Integrated;
        }
        if amd_discrete {
            return GpuKind::Discrete;
        }
        return GpuKind::Discrete;
    }

    if integrated_name || slot_integrated {
        GpuKind::Integrated
    } else {
        GpuKind::Unknown
    }
}

fn is_integrated_slot(slot: &str) -> bool {
    slot.starts_with("0000:00:02") || slot.starts_with("00:02")
}

fn guess_vendor_name(desc: &str) -> String {
    let lower = desc.to_ascii_lowercase();
    if lower.contains("nvidia") {
        "NVIDIA".to_string()
    } else if lower.contains("amd")
        || lower.contains("ati")
        || lower.contains("advanced micro devices")
    {
        "AMD".to_string()
    } else if lower.contains("intel") {
        "Intel".to_string()
    } else {
        "GPU".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lspci_output_detects_integrated() {
        let output = concat!(
            "Slot: \"0000:00:02.0\" Class: \"VGA compatible controller\" ",
            "Vendor: \"Intel Corporation\" Device: \"UHD Graphics 620\"\n",
            "Slot: \"0000:01:00.0\" Class: \"VGA compatible controller\" ",
            "Vendor: \"NVIDIA Corporation\" Device: \"RTX\"\n"
        );
        let gpus = parse_lspci_output(output, true);

        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].kind, GpuKind::Integrated);
    }

    #[test]
    fn classify_amd_rx_as_discrete() {
        let kind = classify_gpu_kind_fields(
            "Advanced Micro Devices, Inc. [AMD/ATI]",
            "Navi 32 [Radeon RX 7700 XT]",
            Some("0000:03:00.0"),
            None,
        );
        assert_eq!(kind, GpuKind::Discrete);
    }
}
