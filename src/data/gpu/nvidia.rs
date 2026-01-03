use std::collections::HashMap;
use std::time::Duration;

use super::types::{GpuInfo, GpuKind, GpuMemory, GpuProcessUsage, GpuTelemetry};
use crate::utils::{mib_to_bytes, run_command_with_timeout};

const NVIDIA_QUERY_BASE: &str = "index,name,memory.used,memory.total,driver_version";
const NVIDIA_QUERY_EXTENDED: &str = concat!(
    "index,name,memory.used,memory.total,utilization.gpu,utilization.memory,temperature.gpu,",
    "power.draw,power.limit,fan.speed,encoder.stats.average,decoder.stats.average,driver_version"
);
const NVIDIA_QUERY_UUID: &str = "index,uuid";
const NVIDIA_QUERY_COMPUTE_APPS: &str = "gpu_uuid,pid,used_memory";

pub fn probe_nvidia_gpus(timeout: Duration) -> Vec<GpuInfo> {
    if let Some(output) = run_command_with_timeout(
        "nvidia-smi",
        &[
            &format!("--query-gpu={NVIDIA_QUERY_EXTENDED}"),
            "--format=csv,noheader,nounits",
        ],
        timeout,
    ) && let Some(gpus) = parse_nvidia_smi_output(&output)
    {
        return gpus;
    }

    let output = run_command_with_timeout(
        "nvidia-smi",
        &[
            &format!("--query-gpu={NVIDIA_QUERY_BASE}"),
            "--format=csv,noheader,nounits",
        ],
        timeout,
    );
    output
        .as_deref()
        .and_then(parse_nvidia_smi_output)
        .unwrap_or_default()
}

pub fn probe_nvidia_processes(timeout: Duration) -> Vec<GpuProcessUsage> {
    let mut by_key: HashMap<(String, u32), GpuProcessUsage> = HashMap::new();

    if let Some(output) = run_command_with_timeout("nvidia-smi", &["pmon", "-c", "1"], timeout) {
        for entry in parse_nvidia_pmon_output(&output) {
            by_key.insert((entry.gpu_id.clone(), entry.pid), entry);
        }
    }

    if let Some(apps_output) = run_command_with_timeout(
        "nvidia-smi",
        &[
            &format!("--query-compute-apps={NVIDIA_QUERY_COMPUTE_APPS}"),
            "--format=csv,noheader,nounits",
        ],
        timeout,
    ) {
        let apps = parse_nvidia_compute_apps_output(&apps_output);
        if !apps.is_empty()
            && let Some(uuid_output) = run_command_with_timeout(
                "nvidia-smi",
                &[
                    &format!("--query-gpu={NVIDIA_QUERY_UUID}"),
                    "--format=csv,noheader,nounits",
                ],
                timeout,
            )
        {
            let uuid_map = parse_nvidia_gpu_uuid_map(&uuid_output);
            if !uuid_map.is_empty() {
                apply_compute_apps_memory(&mut by_key, &uuid_map, apps);
            }
        }
    }

    by_key.into_values().collect()
}

fn parse_nvidia_smi_output(output: &str) -> Option<Vec<GpuInfo>> {
    if output.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut gpus = Vec::new();
    let mut unexpected_format = false;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|part| part.trim()).collect();
        let field_count = parts.len();
        if field_count < 4 {
            unexpected_format = true;
            continue;
        }
        if field_count != 4 && field_count != 5 && field_count < 12 {
            unexpected_format = true;
            continue;
        }

        let driver_version = match field_count {
            5 => parse_optional_string(parts[4]),
            count if count >= 13 => parse_optional_string(parts[12]),
            _ => None,
        };

        let index = match parts[0].parse::<u32>().ok() {
            Some(value) => value,
            None => continue,
        };
        let name = parts[1].to_string();
        let used = match parts[2].parse::<u64>().ok() {
            Some(value) => value,
            None => continue,
        };
        let total = match parts[3].parse::<u64>().ok() {
            Some(value) => value,
            None => continue,
        };

        let telemetry = if field_count >= 12 {
            GpuTelemetry {
                utilization_gpu_pct: parse_optional_f32(parts[4]),
                utilization_mem_pct: parse_optional_f32(parts[5]),
                temperature_c: parse_optional_f32(parts[6]),
                power_draw_w: parse_optional_f32(parts[7]),
                power_limit_w: parse_optional_f32(parts[8]),
                fan_speed_pct: parse_optional_f32(parts[9]),
                encoder_pct: parse_optional_f32(parts[10]),
                decoder_pct: parse_optional_f32(parts[11]),
            }
        } else {
            GpuTelemetry::default()
        };

        gpus.push(GpuInfo {
            id: format!("nvidia:{index}"),
            name: name.clone(),
            vendor: Some("NVIDIA".to_string()),
            device: Some(name),
            driver: Some("nvidia".to_string()),
            driver_version,
            kind: GpuKind::Discrete,
            memory: Some(GpuMemory {
                used_bytes: mib_to_bytes(used),
                total_bytes: mib_to_bytes(total),
            }),
            telemetry,
        });
    }

    if unexpected_format { None } else { Some(gpus) }
}

fn parse_nvidia_pmon_output(output: &str) -> Vec<GpuProcessUsage> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 2 {
                return None;
            }
            let gpu_index = cols[0].parse::<u32>().ok()?;
            let pid = cols[1].parse::<u32>().ok()?;
            if pid == 0 {
                return None;
            }
            let kind = cols.get(2).and_then(|value| parse_optional_char(value));
            let sm_pct = cols.get(3).and_then(|value| parse_optional_f32(value));
            let mem_pct = cols.get(4).and_then(|value| parse_optional_f32(value));
            let enc_pct = cols.get(5).and_then(|value| parse_optional_f32(value));
            let dec_pct = cols.get(6).and_then(|value| parse_optional_f32(value));
            let fb_mb = cols.get(7).and_then(|value| parse_optional_u64(value));

            Some(GpuProcessUsage {
                gpu_id: format!("nvidia:{gpu_index}"),
                pid,
                kind,
                sm_pct,
                mem_pct,
                enc_pct,
                dec_pct,
                fb_mb,
            })
        })
        .collect()
}

fn apply_compute_apps_memory(
    by_key: &mut HashMap<(String, u32), GpuProcessUsage>,
    uuid_map: &HashMap<String, u32>,
    apps: Vec<ComputeAppEntry>,
) {
    for entry in apps {
        let Some(index) = uuid_map.get(&entry.uuid) else {
            continue;
        };
        let gpu_id = format!("nvidia:{index}");
        let key = (gpu_id.clone(), entry.pid);
        by_key
            .entry(key)
            .and_modify(|usage| usage.fb_mb = Some(entry.used_memory_mb))
            .or_insert(GpuProcessUsage {
                gpu_id,
                pid: entry.pid,
                kind: None,
                sm_pct: None,
                mem_pct: None,
                enc_pct: None,
                dec_pct: None,
                fb_mb: Some(entry.used_memory_mb),
            });
    }
}

fn parse_nvidia_gpu_uuid_map(output: &str) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|part| part.trim()).collect();
        if parts.len() < 2 {
            continue;
        }
        let Ok(index) = parts[0].parse::<u32>() else {
            continue;
        };
        let uuid = parts[1];
        if uuid.is_empty() {
            continue;
        }
        map.insert(uuid.to_string(), index);
    }
    map
}

fn parse_nvidia_compute_apps_output(output: &str) -> Vec<ComputeAppEntry> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let parts: Vec<&str> = line.split(',').map(|part| part.trim()).collect();
            if parts.len() < 3 {
                return None;
            }
            let uuid = parts[0].to_string();
            let pid = parts[1].parse::<u32>().ok()?;
            let used_memory_mb = parse_optional_u64(parts[2])?;
            Some(ComputeAppEntry {
                uuid,
                pid,
                used_memory_mb,
            })
        })
        .collect()
}

fn parse_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed.eq_ignore_ascii_case("n/a") {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_optional_f32(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed.eq_ignore_ascii_case("n/a") {
        None
    } else {
        trimmed.parse::<f32>().ok()
    }
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed.eq_ignore_ascii_case("n/a") {
        None
    } else {
        trimmed.parse::<u64>().ok()
    }
}

fn parse_optional_char(value: &str) -> Option<char> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" {
        None
    } else {
        trimmed.chars().next()
    }
}

struct ComputeAppEntry {
    uuid: String,
    pid: u32,
    used_memory_mb: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nvidia_smi_output_parses_multiple_gpus() {
        let output = "0, RTX 3060, 120, 4096\n1, RTX 3070, 0, 8192\n";
        let gpus = parse_nvidia_smi_output(output).unwrap();

        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].id, "nvidia:0");
        assert_eq!(
            gpus[1].memory.as_ref().unwrap().total_bytes,
            mib_to_bytes(8192)
        );
    }

    #[test]
    fn parse_nvidia_smi_output_parses_telemetry() {
        let output = "0, RTX 3060, 120, 4096, 68, 12, 74, 285.5, 320.0, 52, 23, 0\n";
        let gpus = parse_nvidia_smi_output(output).unwrap();

        assert_eq!(gpus.len(), 1);
        let telemetry = &gpus[0].telemetry;
        assert_eq!(telemetry.utilization_gpu_pct, Some(68.0));
        assert_eq!(telemetry.temperature_c, Some(74.0));
        assert_eq!(telemetry.power_draw_w, Some(285.5));
        assert_eq!(telemetry.encoder_pct, Some(23.0));
    }

    #[test]
    fn parse_nvidia_pmon_output_parses_processes() {
        let output = concat!(
            "# gpu        pid  type    sm   mem   enc   dec   fb   bar\n",
            "# Idx        #    C/G     %     %     %     %     MB   MB\n",
            "    0      1234    C     56     12    0     0   400    10\n",
            "    0         0    -      -      -    -     -     -     -\n"
        );

        let processes = parse_nvidia_pmon_output(output);

        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].gpu_id, "nvidia:0");
        assert_eq!(processes[0].pid, 1234);
        assert_eq!(processes[0].kind, Some('C'));
        assert_eq!(processes[0].sm_pct, Some(56.0));
        assert_eq!(processes[0].fb_mb, Some(400));
    }

    #[test]
    fn parse_nvidia_gpu_uuid_map_parses_entries() {
        let output = "0, GPU-aaa\n1, GPU-bbb\n";
        let map = parse_nvidia_gpu_uuid_map(output);

        assert_eq!(map.get("GPU-aaa"), Some(&0));
        assert_eq!(map.get("GPU-bbb"), Some(&1));
    }

    #[test]
    fn parse_nvidia_compute_apps_output_parses_entries() {
        let output = "GPU-aaa, 4242, 1024\n";
        let apps = parse_nvidia_compute_apps_output(output);

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].uuid, "GPU-aaa");
        assert_eq!(apps[0].pid, 4242);
        assert_eq!(apps[0].used_memory_mb, 1024);
    }
}
