use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::lspci::classify_gpu_kind_fields;
use super::types::{GpuInfo, GpuMemory, GpuTelemetry};

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
        let driver_name = driver.as_deref();
        let driver_version = driver_name
            .and_then(read_driver_version)
            .or_else(|| mesa_version_for_driver(driver_name));
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
        let telemetry = read_sysfs_telemetry(&device_path);

        gpus.push(GpuInfo {
            id,
            name: display,
            vendor: Some(vendor),
            device,
            driver,
            driver_version,
            kind,
            memory,
            telemetry,
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

fn read_trimmed<P: AsRef<Path>>(path: P) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let value = raw.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn read_driver_version(driver: &str) -> Option<String> {
    let base = Path::new("/sys/module").join(driver);
    read_trimmed(base.join("version")).or_else(|| read_trimmed(base.join("srcversion")))
}

fn mesa_version_for_driver(driver: Option<&str>) -> Option<String> {
    let driver = driver?;
    if !is_open_source_driver(driver) {
        return None;
    }
    detect_mesa_version()
}

fn is_open_source_driver(driver: &str) -> bool {
    matches!(
        driver,
        "amdgpu" | "radeon" | "i915" | "xe" | "nouveau" | "vmwgfx"
    )
}

fn detect_mesa_version() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        static VERSION: OnceLock<Option<String>> = OnceLock::new();
        VERSION
            .get_or_init(|| {
                mesa_version_from_files()
                    .or_else(mesa_version_from_pkgconfig)
                    .or_else(mesa_version_from_glxinfo)
            })
            .clone()
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

fn mesa_version_from_files() -> Option<String> {
    const PATHS: [&str; 6] = [
        "/usr/share/mesa/mesa.version",
        "/usr/share/mesa/mesa_version",
        "/usr/share/mesa/version",
        "/usr/lib/mesa/mesa.version",
        "/usr/lib/mesa/mesa_version",
        "/usr/lib/mesa/version",
    ];
    for path in PATHS {
        if let Ok(contents) = fs::read_to_string(path) {
            let value = contents.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn mesa_version_from_pkgconfig() -> Option<String> {
    for path in pkg_config_search_paths() {
        let candidate = Path::new(&path).join("mesa.pc");
        if let Some(version) = read_pkgconfig_version(&candidate) {
            return Some(version);
        }
    }
    None
}

fn pkg_config_search_paths() -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(env_paths) = env::var_os("PKG_CONFIG_PATH") {
        for path in env_paths.to_string_lossy().split(':') {
            let path = path.trim();
            if !path.is_empty() {
                paths.push(path.to_string());
            }
        }
    }
    paths.extend(
        [
            "/usr/lib/pkgconfig",
            "/usr/lib64/pkgconfig",
            "/usr/share/pkgconfig",
            "/usr/lib/x86_64-linux-gnu/pkgconfig",
            "/usr/lib/aarch64-linux-gnu/pkgconfig",
            "/usr/lib/arm-linux-gnueabihf/pkgconfig",
            "/usr/local/lib/pkgconfig",
            "/usr/local/lib64/pkgconfig",
            "/usr/local/share/pkgconfig",
        ]
        .iter()
        .map(|path| path.to_string()),
    );
    paths
}

fn read_pkgconfig_version(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("Version:") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn mesa_version_from_glxinfo() -> Option<String> {
    use std::process::Command;
    let output = Command::new("glxinfo").arg("-B").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("OpenGL version string:") {
            // "OpenGL version string: 4.6 (Compatibility Profile) Mesa 25.3.1"
            if let Some(mesa_pos) = line.find("Mesa ") {
                let version_start = mesa_pos + 5;
                let version = line[version_start..].split_whitespace().next()?;
                if !version.is_empty() {
                    return Some(format!("Mesa {}", version));
                }
            }
        }
    }
    None
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

fn read_sysfs_telemetry(device_path: &Path) -> GpuTelemetry {
    let hwmon_dirs = read_hwmon_dirs(device_path);
    let utilization_gpu_pct =
        read_percent_file(device_path, &["gpu_busy_percent", "gt_busy_percent"]);
    let utilization_mem_pct = read_percent_file(device_path, &["mem_busy_percent"]);
    let temperature_c = read_hwmon_temp_c(&hwmon_dirs);
    let fan_speed_pct = read_hwmon_fan_pct(&hwmon_dirs);
    let power_draw_w = read_hwmon_u64(&hwmon_dirs, &["power1_average", "power1_input"])
        .map(|value| value as f32 / 1_000_000.0);
    let power_limit_w = read_hwmon_u64(&hwmon_dirs, &["power1_cap", "power1_cap_max"])
        .map(|value| value as f32 / 1_000_000.0);

    GpuTelemetry {
        utilization_gpu_pct,
        utilization_mem_pct,
        temperature_c,
        power_draw_w,
        power_limit_w,
        fan_speed_pct,
        encoder_pct: None,
        decoder_pct: None,
    }
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

fn read_percent_file(device_path: &Path, names: &[&str]) -> Option<f32> {
    for name in names {
        if let Some(value) = read_u64(device_path.join(name)) {
            return Some((value as f32).clamp(0.0, 100.0));
        }
    }
    None
}

fn read_hwmon_dirs(device_path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = fs::read_dir(device_path.join("hwmon")) else {
        return dirs;
    };
    for entry in entries.flatten() {
        if entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
        {
            dirs.push(entry.path());
        }
    }
    dirs
}

fn read_hwmon_u64(hwmon_dirs: &[PathBuf], names: &[&str]) -> Option<u64> {
    for dir in hwmon_dirs {
        for name in names {
            if let Some(value) = read_u64(dir.join(name)) {
                return Some(value);
            }
        }
    }
    None
}

fn read_hwmon_temp_c(hwmon_dirs: &[PathBuf]) -> Option<f32> {
    let mut max_temp: Option<f32> = None;
    for dir in hwmon_dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("temp") || !name.ends_with("_input") {
                continue;
            }
            if let Some(value) = read_u64(entry.path()) {
                let temp = value as f32 / 1000.0;
                max_temp = Some(max_temp.map(|current| current.max(temp)).unwrap_or(temp));
            }
        }
    }
    max_temp
}

fn read_hwmon_fan_pct(hwmon_dirs: &[PathBuf]) -> Option<f32> {
    for dir in hwmon_dirs {
        if let (Some(speed), Some(max)) = (
            read_u64(dir.join("fan1_input")),
            read_u64(dir.join("fan1_max")),
        ) && max > 0
        {
            return Some((speed as f32 / max as f32) * 100.0);
        }
        if let Some(pwm) = read_u64(dir.join("pwm1")) {
            let max = read_u64(dir.join("pwm1_max")).unwrap_or(255);
            if max > 0 {
                return Some((pwm as f32 / max as f32) * 100.0);
            }
        }
    }
    None
}
