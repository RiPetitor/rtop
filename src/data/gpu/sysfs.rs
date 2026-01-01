use std::fs;
use std::path::Path;

use super::lspci::classify_gpu_kind_fields;
use super::types::{GpuInfo, GpuMemory};

pub fn probe_sysfs_gpus(skip_nvidia: bool) -> Vec<GpuInfo> {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };
    let mut gpus = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device_path = entry.path().join("device");
        let slot = read_link_basename(&device_path);
        let vendor_id = read_hex(device_path.join("vendor"));
        let device_id = read_hex(device_path.join("device"));
        let driver = read_link_basename(device_path.join("driver"));
        let vendor = vendor_name_from_id(vendor_id, driver.as_deref());
        let device = device_id.map(|id| format!("{:04x}", id));
        if skip_nvidia && vendor.eq_ignore_ascii_case("NVIDIA") {
            continue;
        }

        let mut display = vendor.clone();
        if let Some(id) = device_id {
            display = format!("{display} {:04x}", id);
        }
        if let Some(ref driver) = driver {
            display = format!("{display} ({driver})");
        }

        let id = slot
            .as_ref()
            .map(|slot| format!("pci:{slot}"))
            .unwrap_or_else(|| format!("drm:{name}"));
        let kind = classify_gpu_kind_fields(&vendor, &display, slot.as_deref(), vendor_id);
        let memory = read_sysfs_vram(&device_path);

        gpus.push(GpuInfo {
            id,
            name: display,
            vendor: Some(vendor),
            device,
            kind,
            memory,
        });
    }
    gpus
}

fn read_hex<P: AsRef<Path>>(path: P) -> Option<u32> {
    let raw = fs::read_to_string(path).ok()?;
    let trimmed = raw.trim().trim_start_matches("0x");
    u32::from_str_radix(trimmed, 16).ok()
}

fn read_u64<P: AsRef<Path>>(path: P) -> Option<u64> {
    let raw = fs::read_to_string(path).ok()?;
    raw.trim().parse::<u64>().ok()
}

fn read_sysfs_vram(device_path: &Path) -> Option<GpuMemory> {
    let total_bytes = read_u64(device_path.join("mem_info_vram_total"))?;
    if total_bytes == 0 {
        return None;
    }
    let used_bytes = read_u64(device_path.join("mem_info_vram_used")).unwrap_or(0);
    Some(GpuMemory {
        used_bytes,
        total_bytes,
    })
}

fn read_link_basename<P: AsRef<Path>>(path: P) -> Option<String> {
    fs::read_link(path).ok().and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    })
}

fn vendor_name_from_id(vendor_id: Option<u32>, driver: Option<&str>) -> String {
    match vendor_id {
        Some(0x8086) => "Intel".to_string(),
        Some(0x10de) => "NVIDIA".to_string(),
        Some(0x1002) | Some(0x1022) => "AMD".to_string(),
        _ => {
            if let Some(driver) = driver {
                if driver.contains("amdgpu") || driver.contains("radeon") {
                    return "AMD".to_string();
                }
                if driver.contains("i915") {
                    return "Intel".to_string();
                }
                if driver.contains("nouveau") || driver.contains("nvidia") {
                    return "NVIDIA".to_string();
                }
            }
            "GPU".to_string()
        }
    }
}
