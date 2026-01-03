mod actions;
mod containers;
mod gpu;
mod rows;
mod selection;
mod tree;

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use ratatui::prelude::Rect;
use ratatui::widgets::TableState;
use sysinfo::{Pid, System, Uid, Users};

use super::config::Config;
use super::highlight::HighlightMode;
use super::status::{StatusLevel, StatusMessage};
use super::view_mode::ViewMode;
use crate::data::gpu::{GpuInfo, GpuPreference, GpuProcessUsage, GpuSnapshot, start_gpu_monitor};
use crate::data::{ContainerKey, ContainerRow, NetSample, ProcessRow, SortDir, SortKey};

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

    pub fn next(self) -> Self {
        match self {
            GpuProcessSortKey::Pid => GpuProcessSortKey::Kind,
            GpuProcessSortKey::Kind => GpuProcessSortKey::Sm,
            GpuProcessSortKey::Sm => GpuProcessSortKey::Mem,
            GpuProcessSortKey::Mem => GpuProcessSortKey::Enc,
            GpuProcessSortKey::Enc => GpuProcessSortKey::Dec,
            GpuProcessSortKey::Dec => GpuProcessSortKey::Vram,
            GpuProcessSortKey::Vram => GpuProcessSortKey::Name,
            GpuProcessSortKey::Name => GpuProcessSortKey::Pid,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            GpuProcessSortKey::Pid => GpuProcessSortKey::Name,
            GpuProcessSortKey::Kind => GpuProcessSortKey::Pid,
            GpuProcessSortKey::Sm => GpuProcessSortKey::Kind,
            GpuProcessSortKey::Mem => GpuProcessSortKey::Sm,
            GpuProcessSortKey::Enc => GpuProcessSortKey::Mem,
            GpuProcessSortKey::Dec => GpuProcessSortKey::Enc,
            GpuProcessSortKey::Vram => GpuProcessSortKey::Dec,
            GpuProcessSortKey::Name => GpuProcessSortKey::Vram,
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
        rows::merge_optional_max(&mut self.sm_pct, entry.sm_pct);
        rows::merge_optional_max(&mut self.mem_pct, entry.mem_pct);
        rows::merge_optional_max(&mut self.enc_pct, entry.enc_pct);
        rows::merge_optional_max(&mut self.dec_pct, entry.dec_pct);
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
