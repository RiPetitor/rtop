use std::time::Duration;

use super::types::{GpuInfo, GpuKind, GpuMemory};
use crate::utils::{mib_to_bytes, run_command_with_timeout};

pub fn probe_nvidia_gpus(timeout: Duration) -> Vec<GpuInfo> {
    let output = run_command_with_timeout(
        "nvidia-smi",
        &[
            "--query-gpu=index,name,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ],
        timeout,
    );
    output
        .as_deref()
        .map(parse_nvidia_smi_output)
        .unwrap_or_default()
}

fn parse_nvidia_smi_output(output: &str) -> Vec<GpuInfo> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(',').map(|part| part.trim());
            let index = parts.next()?.parse::<u32>().ok()?;
            let name = parts.next()?.to_string();
            let used = parts.next()?.parse::<u64>().ok()?;
            let total = parts.next()?.parse::<u64>().ok()?;
            Some(GpuInfo {
                id: format!("nvidia:{index}"),
                name: name.clone(),
                vendor: Some("NVIDIA".to_string()),
                device: Some(name),
                kind: GpuKind::Discrete,
                memory: Some(GpuMemory {
                    used_bytes: mib_to_bytes(used),
                    total_bytes: mib_to_bytes(total),
                }),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nvidia_smi_output_parses_multiple_gpus() {
        let output = "0, RTX 3060, 120, 4096\n1, RTX 3070, 0, 8192\n";
        let gpus = parse_nvidia_smi_output(output);

        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].id, "nvidia:0");
        assert_eq!(
            gpus[1].memory.as_ref().unwrap().total_bytes,
            mib_to_bytes(8192)
        );
    }
}
