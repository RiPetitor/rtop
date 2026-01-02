use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::time::Instant;

use ratatui::prelude::Rect;
use ratatui::widgets::TableState;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System, Uid, Users};

use super::config::Config;
use super::highlight::HighlightMode;
use super::status::{StatusLevel, StatusMessage};
use super::view_mode::ViewMode;
use crate::data::gpu::{
    GpuInfo, GpuPreference, GpuProcessUsage, GpuSnapshot, default_gpu_index, start_gpu_monitor,
};
use crate::data::{
    ContainerKey, ContainerRow, NetSample, ProcessRow, SortDir, SortKey, container_key_for_pid,
    net_sample_for_pid, netns_id_for_pid, sort_process_rows,
};

pub struct ConfirmKill {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub start_time: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Russian",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" => Some(Language::English),
            "ru" | "rus" | "russian" => Some(Language::Russian),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Language::English => Language::Russian,
            Language::Russian => Language::English,
        }
    }
}

#[derive(Clone, Copy)]
pub struct HeaderRegion {
    pub key: SortKey,
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuProcessSortKey {
    Pid,
    Kind,
    Sm,
    Mem,
    Enc,
    Dec,
    Vram,
    Name,
}

impl GpuProcessSortKey {
    pub fn default_dir(self) -> SortDir {
        match self {
            GpuProcessSortKey::Pid | GpuProcessSortKey::Kind | GpuProcessSortKey::Name => {
                SortDir::Asc
            }
            GpuProcessSortKey::Sm
            | GpuProcessSortKey::Mem
            | GpuProcessSortKey::Enc
            | GpuProcessSortKey::Dec
            | GpuProcessSortKey::Vram => SortDir::Desc,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GpuProcessHeaderRegion {
    pub key: GpuProcessSortKey,
    pub rect: Rect,
}

#[derive(Default, Clone, Copy)]
struct ProcessGpuUsage {
    sm_pct: Option<f32>,
    mem_pct: Option<f32>,
    enc_pct: Option<f32>,
    dec_pct: Option<f32>,
    fb_bytes: u64,
    kind: Option<char>,
}

struct NetSampleEntry {
    sample: NetSample,
    timestamp: Instant,
}

impl ProcessGpuUsage {
    fn apply_entry(&mut self, entry: &GpuProcessUsage) {
        merge_optional_max(&mut self.sm_pct, entry.sm_pct);
        merge_optional_max(&mut self.mem_pct, entry.mem_pct);
        merge_optional_max(&mut self.enc_pct, entry.enc_pct);
        merge_optional_max(&mut self.dec_pct, entry.dec_pct);
        if let Some(fb_mb) = entry.fb_mb {
            self.fb_bytes = self
                .fb_bytes
                .saturating_add(fb_mb.saturating_mul(1024 * 1024));
        }
        if let Some(kind) = entry.kind {
            match self.kind {
                Some('C') => {}
                Some('G') if kind == 'C' => self.kind = Some('C'),
                None => self.kind = Some(kind),
                _ => {}
            }
        }
    }
}

pub struct App {
    pub system: System,
    users: Users,
    current_user_id: Option<Uid>,
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
    pub tree_view: bool,
    pub rows: Vec<ProcessRow>,
    pub table_state: TableState,
    pub selected_pid: Option<u32>,
    pub scroll: usize,
    pub tree_labels: HashMap<u32, String>,
    pub process_body: Option<Rect>,
    pub process_header_regions: Vec<HeaderRegion>,
    pub gpu_process_header_regions: Vec<GpuProcessHeaderRegion>,
    pub gpu_process_body: Option<Rect>,
    pub gpu_process_order: Vec<u32>,
    pub gpu_process_scroll: usize,
    pub gpu_process_sort_key: GpuProcessSortKey,
    pub gpu_process_sort_dir: SortDir,
    pub container_rows: Vec<ContainerRow>,
    pub container_table_state: TableState,
    pub container_selected: Option<ContainerKey>,
    pub container_scroll: usize,
    pub container_pid_map: HashMap<u32, ContainerKey>,
    pub container_filter: Option<ContainerKey>,
    container_net_prev: HashMap<u64, NetSampleEntry>,
    pub confirm: Option<ConfirmKill>,
    pub highlight_mode: HighlightMode,
    pub vram_enabled: bool,
    pub gpu_pref: GpuPreference,
    pub gpu_list: Vec<GpuInfo>,
    pub gpu_selected: Option<String>,
    pub gpu_processes: Vec<GpuProcessUsage>,
    gpu_rx: Option<mpsc::Receiver<GpuSnapshot>>,
    pub status: Option<StatusMessage>,
    pub view_mode: ViewMode,
    pub show_setup: bool,
    pub show_help: bool,
    pub language: Language,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let users = Users::new_with_refreshed_list();
        let current_user_id = system
            .process(Pid::from_u32(std::process::id()))
            .and_then(|process| process.user_id())
            .cloned();
        let gpu_rx = if config.vram_enabled {
            Some(start_gpu_monitor(config.gpu_poll_rate))
        } else {
            None
        };
        let mut app = Self {
            system,
            users,
            current_user_id,
            sort_key: config.sort_key,
            sort_dir: config.sort_dir,
            tree_view: false,
            rows: Vec::new(),
            table_state: TableState::default(),
            selected_pid: None,
            scroll: 0,
            tree_labels: HashMap::new(),
            process_body: None,
            process_header_regions: Vec::new(),
            gpu_process_header_regions: Vec::new(),
            gpu_process_body: None,
            gpu_process_order: Vec::new(),
            gpu_process_scroll: 0,
            gpu_process_sort_key: GpuProcessSortKey::Sm,
            gpu_process_sort_dir: GpuProcessSortKey::Sm.default_dir(),
            container_rows: Vec::new(),
            container_table_state: TableState::default(),
            container_selected: None,
            container_scroll: 0,
            container_pid_map: HashMap::new(),
            container_filter: None,
            container_net_prev: HashMap::new(),
            confirm: None,
            highlight_mode: HighlightMode::default(),
            vram_enabled: config.vram_enabled,
            gpu_pref: config.gpu_pref,
            gpu_list: Vec::new(),
            gpu_selected: None,
            gpu_processes: Vec::new(),
            gpu_rx,
            status: None,
            view_mode: ViewMode::default(),
            show_setup: false,
            show_help: false,
            language: config.language,
        };
        app.update_rows();
        app.poll_gpu_updates();
        app
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.users.refresh();
        self.update_rows();
        let needs_containers =
            matches!(self.view_mode, ViewMode::Container) || self.container_filter.is_some();
        if needs_containers {
            self.update_containers();
            if let Some(filter) = self.container_filter.as_ref() {
                self.rows
                    .retain(|row| self.container_pid_map.get(&row.pid) == Some(filter));
                self.sync_selection();
            }
        }
    }

    pub fn tick(&mut self) {
        self.poll_gpu_updates();
        self.clear_expired_status();
    }

    pub fn set_sort_key(&mut self, key: SortKey) {
        if self.tree_view && key != SortKey::Pid {
            return;
        }
        self.sort_key = key;
        self.sort_dir = key.default_dir();
        self.update_rows();
    }

    pub fn toggle_sort_dir(&mut self) {
        if self.tree_view {
            return;
        }
        self.sort_dir = self.sort_dir.toggle();
        self.update_rows();
    }

    pub fn cycle_highlight_mode(&mut self) {
        self.highlight_mode = self.highlight_mode.cycle();
    }

    pub fn current_user_name(&self) -> Option<&str> {
        let user_id = self.current_user_id.as_ref()?;
        self.users.get_user_by_id(user_id).map(|user| user.name())
    }

    pub fn update_rows(&mut self) {
        let gpu_usage = build_gpu_usage_map(&self.gpu_processes);
        let current_user_id = self.current_user_id.as_ref();
        let mut rows_map = HashMap::with_capacity(self.system.processes().len());
        let mut parents = HashMap::with_capacity(self.system.processes().len());

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
            let is_gui = is_gui_process(process.environ());

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
            let layout = build_tree_layout(&parents, &rows_map);
            let mut rows = Vec::with_capacity(rows_map.len());
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
        self.sync_selection();
    }

    fn sync_selection(&mut self) {
        if self.rows.is_empty() {
            self.table_state.select(None);
            self.selected_pid = None;
            self.scroll = 0;
            return;
        }

        let selected_idx = self
            .selected_pid
            .and_then(|pid| self.rows.iter().position(|row| row.pid == pid))
            .or_else(|| self.table_state.selected())
            .filter(|&idx| idx < self.rows.len())
            .unwrap_or(0);

        self.table_state.select(Some(selected_idx));
        self.selected_pid = Some(self.rows[selected_idx].pid);
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

    pub fn move_selection(&mut self, delta: i32) {
        if self.rows.is_empty() {
            self.table_state.select(None);
            self.selected_pid = None;
            return;
        }

        let current = self.table_state.selected().unwrap_or(0);
        let len = self.rows.len();
        let new_index = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (current + delta as usize).min(len.saturating_sub(1))
        };

        self.table_state.select(Some(new_index));
        self.selected_pid = Some(self.rows[new_index].pid);
    }

    pub fn move_gpu_process_selection(&mut self, delta: i32) {
        let len = self.gpu_process_order.len();
        if len == 0 {
            return;
        }

        let current = self
            .selected_pid
            .and_then(|pid| {
                self.gpu_process_order
                    .iter()
                    .position(|&entry| entry == pid)
            })
            .unwrap_or(0);

        let new_index = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (current + delta as usize).min(len.saturating_sub(1))
        };

        self.selected_pid = Some(self.gpu_process_order[new_index]);
        let max_rows = self
            .gpu_process_body
            .map(|rect| rect.height as usize)
            .unwrap_or(0);
        if max_rows > 0 {
            self.ensure_gpu_process_visible(max_rows);
        }
    }

    pub fn select_gpu_process_first(&mut self) {
        if let Some(pid) = self.gpu_process_order.first().copied() {
            self.selected_pid = Some(pid);
        }
        let max_rows = self
            .gpu_process_body
            .map(|rect| rect.height as usize)
            .unwrap_or(0);
        if max_rows > 0 {
            self.ensure_gpu_process_visible(max_rows);
        }
    }

    pub fn select_gpu_process_last(&mut self) {
        if let Some(pid) = self.gpu_process_order.last().copied() {
            self.selected_pid = Some(pid);
        }
        let max_rows = self
            .gpu_process_body
            .map(|rect| rect.height as usize)
            .unwrap_or(0);
        if max_rows > 0 {
            self.ensure_gpu_process_visible(max_rows);
        }
    }

    pub fn select_process_row(&mut self, index: usize) {
        if self.rows.is_empty() {
            self.table_state.select(None);
            self.selected_pid = None;
            return;
        }

        let idx = index.min(self.rows.len().saturating_sub(1));
        self.table_state.select(Some(idx));
        self.selected_pid = Some(self.rows[idx].pid);
    }

    pub fn select_process_pid(&mut self, pid: u32) {
        self.selected_pid = Some(pid);
        if let Some(index) = self.rows.iter().position(|row| row.pid == pid) {
            self.table_state.select(Some(index));
        }
    }

    pub fn selected_row(&self) -> Option<&ProcessRow> {
        self.table_state
            .selected()
            .and_then(|idx| self.rows.get(idx))
    }

    pub fn ensure_visible(&mut self, max_rows: usize) {
        if max_rows == 0 {
            return;
        }
        if let Some(selected) = self.table_state.selected() {
            if selected < self.scroll {
                self.scroll = selected;
            } else if selected >= self.scroll + max_rows {
                self.scroll = selected + 1 - max_rows;
            }
        }
        let max_scroll = self.rows.len().saturating_sub(max_rows);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub fn ensure_gpu_process_visible(&mut self, max_rows: usize) {
        if max_rows == 0 {
            return;
        }
        if self.gpu_process_order.is_empty() {
            self.gpu_process_scroll = 0;
            return;
        }

        if let Some(selected) = self.selected_pid.and_then(|pid| {
            self.gpu_process_order
                .iter()
                .position(|&entry| entry == pid)
        }) {
            if selected < self.gpu_process_scroll {
                self.gpu_process_scroll = selected;
            } else if selected >= self.gpu_process_scroll + max_rows {
                self.gpu_process_scroll = selected + 1 - max_rows;
            }
        }

        let max_scroll = self.gpu_process_order.len().saturating_sub(max_rows);
        if self.gpu_process_scroll > max_scroll {
            self.gpu_process_scroll = max_scroll;
        }
    }

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
        self.set_view_mode(ViewMode::Processes);
        self.refresh();
    }

    pub fn exit_container_drill(&mut self) {
        if self.container_filter.is_none() {
            return;
        }
        self.container_filter = None;
        self.set_view_mode(ViewMode::Container);
        self.refresh();
    }

    pub fn open_confirm(&mut self) {
        if let Some(row) = self.selected_row() {
            self.open_confirm_for_pid(row.pid);
        }
    }

    pub fn open_confirm_for_pid(&mut self, pid: u32) {
        if let Some(row) = self.rows.iter().find(|row| row.pid == pid) {
            self.confirm = Some(ConfirmKill {
                pid: row.pid,
                name: row.name.clone(),
                cpu: row.cpu,
                mem_bytes: row.mem_bytes,
                status: row.status.clone(),
                start_time: row.start_time,
            });
            return;
        }

        let pid = Pid::from_u32(pid);
        let Some(process) = self.system.process(pid) else {
            self.set_status(
                StatusLevel::Warn,
                format!("Process PID {} not found", pid.as_u32()),
            );
            return;
        };

        self.confirm = Some(ConfirmKill {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cpu: process.cpu_usage(),
            mem_bytes: process.memory(),
            status: format!("{:?}", process.status()),
            start_time: process.start_time(),
        });
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
    }

    pub fn confirm_kill(&mut self) {
        if let Some(confirm) = self.confirm.take() {
            let pid = Pid::from_u32(confirm.pid);
            self.system
                .refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
            if let Some(process) = self.system.process(pid) {
                if process.start_time() != confirm.start_time {
                    self.set_status(
                        StatusLevel::Warn,
                        format!("PID {} reused; refusing SIGTERM", confirm.pid),
                    );
                } else {
                    match process.kill_with(Signal::Term) {
                        Some(true) => self.set_status(
                            StatusLevel::Info,
                            format!("Sent SIGTERM to PID {}", confirm.pid),
                        ),
                        Some(false) => self.set_status(
                            StatusLevel::Warn,
                            format!("Failed to send SIGTERM to PID {}", confirm.pid),
                        ),
                        None => self.set_status(
                            StatusLevel::Warn,
                            format!("SIGTERM not supported for PID {}", confirm.pid),
                        ),
                    }
                }
            } else {
                self.set_status(
                    StatusLevel::Warn,
                    format!("Process PID {} not found", confirm.pid),
                );
            }
            self.refresh();
        }
    }

    fn poll_gpu_updates(&mut self) {
        let Some(rx) = self.gpu_rx.as_ref() else {
            return;
        };
        let mut latest = None;
        while let Ok(snapshot) = rx.try_recv() {
            latest = Some(snapshot);
        }
        if let Some(snapshot) = latest {
            self.update_gpu_list(snapshot.gpus);
            self.gpu_processes = snapshot.processes;
        }
    }

    fn update_gpu_list(&mut self, mut gpus: Vec<GpuInfo>) {
        gpus.sort_by_key(|gpu| gpu.kind.sort_rank());
        self.gpu_list = gpus;
        self.sync_gpu_selection();
    }

    fn sync_gpu_selection(&mut self) {
        if self.gpu_list.is_empty() {
            self.gpu_selected = None;
            return;
        }

        if let Some(selected) = self.gpu_selected.as_ref()
            && self.gpu_list.iter().any(|gpu| &gpu.id == selected)
        {
            return;
        }

        if let Some(idx) = default_gpu_index(&self.gpu_list, self.gpu_pref) {
            self.gpu_selected = Some(self.gpu_list[idx].id.clone());
        }
    }

    pub fn select_next_gpu(&mut self) {
        if self.gpu_list.is_empty() {
            return;
        }
        let current = self.selected_gpu_index().unwrap_or(0);
        let next = (current + 1) % self.gpu_list.len();
        self.gpu_selected = Some(self.gpu_list[next].id.clone());
    }

    pub fn select_prev_gpu(&mut self) {
        if self.gpu_list.is_empty() {
            return;
        }
        let current = self.selected_gpu_index().unwrap_or(0);
        let next = if current == 0 {
            self.gpu_list.len() - 1
        } else {
            current - 1
        };
        self.gpu_selected = Some(self.gpu_list[next].id.clone());
    }

    pub fn selected_gpu(&self) -> Option<(usize, &GpuInfo)> {
        let idx = self.selected_gpu_index()?;
        self.gpu_list.get(idx).map(|gpu| (idx, gpu))
    }

    fn selected_gpu_index(&self) -> Option<usize> {
        let selected = self.gpu_selected.as_ref()?;
        self.gpu_list.iter().position(|gpu| &gpu.id == selected)
    }

    pub fn set_status(&mut self, level: StatusLevel, message: String) {
        self.status = Some(StatusMessage::new(level, message));
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        if mode != ViewMode::Processes && mode != ViewMode::Overview {
            self.container_filter = None;
            self.tree_view = false;
        }
        self.view_mode = mode;
    }

    pub fn cycle_view_mode(&mut self) {
        let next = match self.view_mode {
            ViewMode::Overview => ViewMode::Processes,
            ViewMode::Processes => ViewMode::GpuFocus,
            ViewMode::GpuFocus => ViewMode::SystemInfo,
            ViewMode::SystemInfo => ViewMode::Container,
            ViewMode::Container => ViewMode::Overview,
        };
        if next != ViewMode::Processes && next != ViewMode::Overview {
            self.container_filter = None;
            self.tree_view = false;
        }
        self.view_mode = next;
    }

    pub fn toggle_tree_view(&mut self) {
        if self.view_mode != ViewMode::Processes && self.view_mode != ViewMode::Overview {
            return;
        }
        self.tree_view = !self.tree_view;
        if self.tree_view {
            self.sort_key = SortKey::Pid;
            self.sort_dir = SortDir::Asc;
        }
        self.update_rows();
    }

    pub fn toggle_setup(&mut self) {
        self.show_setup = !self.show_setup;
        if self.show_setup {
            self.show_help = false;
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.show_setup = false;
        }
    }

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        if let Err(err) = super::config::save_language_preference(self.language) {
            self.set_status(StatusLevel::Warn, format!("Failed to save language: {err}"));
        }
    }

    pub fn sort_key_for_header_click(&self, column: u16, row: u16) -> Option<SortKey> {
        self.process_header_regions
            .iter()
            .find(|region| {
                row >= region.rect.y
                    && row < region.rect.y.saturating_add(region.rect.height)
                    && column >= region.rect.x
                    && column < region.rect.x.saturating_add(region.rect.width)
            })
            .map(|region| region.key)
    }

    pub fn gpu_sort_key_for_header_click(
        &self,
        column: u16,
        row: u16,
    ) -> Option<GpuProcessSortKey> {
        self.gpu_process_header_regions
            .iter()
            .find(|region| {
                row >= region.rect.y
                    && row < region.rect.y.saturating_add(region.rect.height)
                    && column >= region.rect.x
                    && column < region.rect.x.saturating_add(region.rect.width)
            })
            .map(|region| region.key)
    }

    pub fn set_gpu_process_sort_key(&mut self, key: GpuProcessSortKey) {
        self.gpu_process_sort_key = key;
        self.gpu_process_sort_dir = key.default_dir();
    }

    pub fn toggle_gpu_process_sort_dir(&mut self) {
        self.gpu_process_sort_dir = self.gpu_process_sort_dir.toggle();
    }

    pub fn selected_gpu_process_pid(&self) -> Option<u32> {
        let pid = self.selected_pid?;
        let selected_id = self.selected_gpu().map(|(_, gpu)| gpu.id.as_str())?;
        let has_pid = self
            .gpu_processes
            .iter()
            .any(|entry| entry.pid == pid && entry.gpu_id == selected_id);
        has_pid.then_some(pid)
    }

    fn clear_expired_status(&mut self) {
        if let Some(status) = self.status.as_ref()
            && status.is_expired()
        {
            self.status = None;
        }
    }

    /// Apply GPU snapshot from event system
    pub fn apply_gpu_snapshot(&mut self, snapshot: crate::data::gpu::GpuSnapshot) {
        self.update_gpu_list(snapshot.gpus);
        self.gpu_processes = snapshot.processes;
    }
}

fn build_gpu_usage_map(gpu_processes: &[GpuProcessUsage]) -> HashMap<u32, ProcessGpuUsage> {
    let mut map = HashMap::with_capacity(gpu_processes.len());
    for entry in gpu_processes {
        map.entry(entry.pid)
            .or_insert_with(ProcessGpuUsage::default)
            .apply_entry(entry);
    }
    map
}

struct TreeLayout {
    order: Vec<u32>,
    labels: HashMap<u32, String>,
}

fn build_tree_layout(
    parents: &HashMap<u32, Option<u32>>,
    rows: &HashMap<u32, ProcessRow>,
) -> TreeLayout {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, parent) in parents.iter() {
        if let Some(parent) = *parent {
            children.entry(parent).or_default().push(pid);
        }
    }
    for list in children.values_mut() {
        list.sort_unstable();
    }

    let mut roots = Vec::new();
    for (&pid, parent) in parents.iter() {
        let has_parent = parent
            .and_then(|parent| parents.contains_key(&parent).then_some(parent))
            .is_some();
        if !has_parent {
            roots.push(pid);
        }
    }
    roots.sort_unstable();

    let mut layout = TreeLayout {
        order: Vec::with_capacity(rows.len()),
        labels: HashMap::with_capacity(rows.len()),
    };
    let mut visited = HashSet::with_capacity(rows.len());

    for (idx, root) in roots.iter().enumerate() {
        let is_last = idx + 1 == roots.len();
        push_tree_layout(
            *root,
            "",
            is_last,
            true,
            &children,
            rows,
            &mut layout,
            &mut visited,
        );
    }

    layout
}

#[allow(clippy::too_many_arguments)]
fn push_tree_layout(
    pid: u32,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    children: &HashMap<u32, Vec<u32>>,
    rows: &HashMap<u32, ProcessRow>,
    layout: &mut TreeLayout,
    visited: &mut HashSet<u32>,
) {
    if !visited.insert(pid) {
        return;
    }
    let Some(row) = rows.get(&pid) else {
        return;
    };

    let connector = if is_root {
        ""
    } else if is_last {
        "\\- "
    } else {
        "|- "
    };
    let label = format!("{prefix}{connector}{}", row.name);
    layout.labels.insert(pid, label);
    layout.order.push(pid);

    let next_prefix = if is_root {
        String::new()
    } else if is_last {
        format!("{prefix}   ")
    } else {
        format!("{prefix}|  ")
    };

    if let Some(list) = children.get(&pid) {
        let last_index = list.len().saturating_sub(1);
        for (idx, child) in list.iter().enumerate() {
            push_tree_layout(
                *child,
                &next_prefix,
                idx == last_index,
                false,
                children,
                rows,
                layout,
                visited,
            );
        }
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
