use std::collections::HashMap;

use sysinfo::Uid;

use super::{App, ProcessGpuUsage};
use crate::data::gpu::GpuProcessUsage;
use crate::data::{ProcessRow, sort_process_rows};

fn build_gpu_usage_map(gpu_processes: &[GpuProcessUsage]) -> HashMap<u32, ProcessGpuUsage> {
    let mut map = HashMap::with_capacity(gpu_processes.len());
    for entry in gpu_processes {
        map.entry(entry.pid)
            .or_insert_with(ProcessGpuUsage::default)
            .apply_entry(entry);
    }
    map
}

pub(super) fn merge_optional_max(current: &mut Option<f32>, incoming: Option<f32>) {
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

impl App {
    pub fn update_rows(&mut self) {
        let gpu_usage = build_gpu_usage_map(&self.gpu_processes);
        let current_user_id = self.current_user_id.as_ref();
        let mut rows_map = HashMap::with_capacity(self.system.processes().len());
        let mut parents = HashMap::with_capacity(self.system.processes().len());

        // Collect current PIDs for cache cleanup
        let current_pids: std::collections::HashSet<u32> = self
            .system
            .processes()
            .keys()
            .map(|pid| pid.as_u32())
            .collect();

        for (pid, process) in self.system.processes() {
            let pid = pid.as_u32();
            let user_id = process.user_id();
            let user = user_id
                .and_then(|id| self.users.get_user_by_id(id))
                .map(|user| user.name().to_string());
            let is_current_user = match (current_user_id, user_id) {
                (Some(current), Some(id)) => current == id,
                _ => false,
            };
            let is_non_root = is_non_root_user(user_id);
            // Use cached GUI detection result or compute and cache it
            let is_gui = *self
                .gui_process_cache
                .entry(pid)
                .or_insert_with(|| is_gui_process(process.environ()));

            parents.insert(pid, process.parent().map(|parent| parent.as_u32()));

            rows_map.insert(
                pid,
                ProcessRow {
                    pid,
                    user,
                    name: process.name().to_string_lossy().into_owned(),
                    cpu: process.cpu_usage(),
                    mem_bytes: process.memory(),
                    status: format!("{:?}", process.status()),
                    start_time: process.start_time(),
                    uptime_secs: process.run_time(),
                    is_current_user,
                    is_non_root,
                    is_gui,
                    gpu_sm_pct: gpu_usage.get(&pid).and_then(|usage| usage.sm_pct),
                    gpu_mem_pct: gpu_usage.get(&pid).and_then(|usage| usage.mem_pct),
                    gpu_enc_pct: gpu_usage.get(&pid).and_then(|usage| usage.enc_pct),
                    gpu_dec_pct: gpu_usage.get(&pid).and_then(|usage| usage.dec_pct),
                    gpu_fb_bytes: gpu_usage
                        .get(&pid)
                        .and_then(|usage| (usage.fb_bytes > 0).then_some(usage.fb_bytes)),
                    gpu_kind: gpu_usage.get(&pid).and_then(|usage| usage.kind),
                },
            );
        }

        if self.tree_view {
            let layout = super::tree::build_tree_layout(&parents, &rows_map);
            let mut rows = Vec::with_capacity(rows_map.len());
            let mut rows_map = rows_map;
            for pid in layout.order {
                if let Some(row) = rows_map.remove(&pid) {
                    rows.push(row);
                }
            }
            if !rows_map.is_empty() {
                let mut extras = rows_map.into_values().collect::<Vec<_>>();
                extras.sort_by_key(|row| row.pid);
                rows.extend(extras);
            }
            self.rows = rows;
            self.tree_labels = layout.labels;
        } else {
            let mut rows = rows_map.into_values().collect::<Vec<_>>();
            sort_process_rows(&mut rows, self.sort_key, self.sort_dir);
            self.rows = rows;
            self.tree_labels.clear();
        }

        let filter = self.process_filter.trim();
        if !filter.is_empty() {
            let needle = filter.to_lowercase();
            self.rows
                .retain(|row| row.name.to_lowercase().contains(&needle));
        }

        // Clean up GUI cache for dead processes
        self.gui_process_cache
            .retain(|pid, _| current_pids.contains(pid));

        self.sync_selection();
    }
}

fn is_gui_process(environ: &[std::ffi::OsString]) -> bool {
    environ.iter().any(|entry| {
        let s = entry.to_string_lossy();
        s.starts_with("DISPLAY=")
            || s.starts_with("WAYLAND_DISPLAY=")
            || s.starts_with("MIR_SOCKET=")
    })
}

fn is_non_root_user(user_id: Option<&Uid>) -> bool {
    #[cfg(unix)]
    {
        use std::ops::Deref;

        user_id.map(|id| *id.deref() != 0).unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        let _ = user_id;
        false
    }
}
