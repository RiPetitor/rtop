use std::collections::HashMap;
use std::time::Instant;

use super::{App, NetSampleEntry};
use crate::data::{
    ContainerKey, ContainerRow, container_key_for_pid, net_sample_for_pid, netns_id_for_pid,
};

impl App {
    pub fn update_containers(&mut self) {
        #[derive(Default)]
        struct ContainerUsage {
            cpu: f32,
            mem_bytes: u64,
            proc_count: usize,
            netns_id: Option<u64>,
        }

        let mut map: HashMap<ContainerKey, ContainerUsage> = HashMap::new();
        let mut pid_map = HashMap::new();
        let mut netns_pids: HashMap<u64, u32> = HashMap::new();
        let mut netns_container_counts: HashMap<u64, usize> = HashMap::new();
        for (pid, process) in self.system.processes() {
            let pid = pid.as_u32();
            if let Some(key) = container_key_for_pid(pid) {
                pid_map.insert(pid, key.clone());
                let entry = map.entry(key.clone()).or_default();
                entry.cpu += process.cpu_usage();
                entry.mem_bytes = entry.mem_bytes.saturating_add(process.memory());
                entry.proc_count += 1;
                if entry.netns_id.is_none()
                    && let Some(netns_id) = netns_id_for_pid(pid)
                {
                    entry.netns_id = Some(netns_id);
                    netns_pids.entry(netns_id).or_insert(pid);
                    *netns_container_counts.entry(netns_id).or_insert(0) += 1;
                }
            }
        }

        let now = Instant::now();
        let mut net_rates: HashMap<u64, u64> = HashMap::new();
        let mut next_net_prev: HashMap<u64, NetSampleEntry> = HashMap::new();
        for (netns_id, pid) in netns_pids {
            if let Some(sample) = net_sample_for_pid(pid) {
                if let Some(prev) = self.container_net_prev.get(&netns_id) {
                    let elapsed = now.duration_since(prev.timestamp).as_secs_f64();
                    if elapsed > 0.0 {
                        let rx_delta = sample.rx_bytes.saturating_sub(prev.sample.rx_bytes);
                        let tx_delta = sample.tx_bytes.saturating_sub(prev.sample.tx_bytes);
                        let rx_rate = (rx_delta as f64 / elapsed).round() as u64;
                        let tx_rate = (tx_delta as f64 / elapsed).round() as u64;
                        net_rates.insert(netns_id, rx_rate.saturating_add(tx_rate));
                    }
                }
                next_net_prev.insert(
                    netns_id,
                    NetSampleEntry {
                        sample,
                        timestamp: now,
                    },
                );
            }
        }
        self.container_net_prev = next_net_prev;

        let mut rows = map
            .into_iter()
            .map(|(key, usage)| {
                let net_bytes_per_sec = usage.netns_id.and_then(|netns_id| {
                    let count = netns_container_counts.get(&netns_id).copied().unwrap_or(0);
                    if count > 1 {
                        None
                    } else {
                        net_rates.get(&netns_id).copied()
                    }
                });
                ContainerRow::new(
                    key,
                    usage.cpu,
                    usage.mem_bytes,
                    usage.proc_count,
                    net_bytes_per_sec,
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.mem_bytes.cmp(&a.mem_bytes))
                .then_with(|| a.label.cmp(&b.label))
        });

        self.container_rows = rows;
        self.container_pid_map = pid_map;
        self.sync_container_selection();
    }

    fn sync_container_selection(&mut self) {
        if self.container_rows.is_empty() {
            self.container_table_state.select(None);
            self.container_selected = None;
            self.container_scroll = 0;
            return;
        }

        let selected_idx = self
            .container_selected
            .as_ref()
            .and_then(|key| self.container_rows.iter().position(|row| &row.key == key))
            .or_else(|| self.container_table_state.selected())
            .filter(|&idx| idx < self.container_rows.len())
            .unwrap_or(0);

        self.container_table_state.select(Some(selected_idx));
        self.container_selected = Some(self.container_rows[selected_idx].key.clone());
    }

    pub fn move_container_selection(&mut self, delta: i32) {
        if self.container_rows.is_empty() {
            self.container_table_state.select(None);
            self.container_selected = None;
            return;
        }

        let current = self.container_table_state.selected().unwrap_or(0);
        let len = self.container_rows.len();
        let new_index = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (current + delta as usize).min(len.saturating_sub(1))
        };

        self.container_table_state.select(Some(new_index));
        self.container_selected = Some(self.container_rows[new_index].key.clone());
    }

    pub fn selected_container(&self) -> Option<&ContainerRow> {
        self.container_table_state
            .selected()
            .and_then(|idx| self.container_rows.get(idx))
    }

    pub fn ensure_container_visible(&mut self, max_rows: usize) {
        if max_rows == 0 {
            return;
        }
        if let Some(selected) = self.container_table_state.selected() {
            if selected < self.container_scroll {
                self.container_scroll = selected;
            } else if selected >= self.container_scroll + max_rows {
                self.container_scroll = selected + 1 - max_rows;
            }
        }
        let max_scroll = self.container_rows.len().saturating_sub(max_rows);
        if self.container_scroll > max_scroll {
            self.container_scroll = max_scroll;
        }
    }

    pub fn enter_container(&mut self) {
        let Some(row) = self.selected_container() else {
            return;
        };
        self.container_filter = Some(row.key.clone());
        self.set_view_mode(super::ViewMode::Processes);
        self.refresh();
    }

    pub fn exit_container_drill(&mut self) {
        if self.container_filter.is_none() {
            return;
        }
        self.container_filter = None;
        self.set_view_mode(super::ViewMode::Container);
        self.refresh();
    }
}
