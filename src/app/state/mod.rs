mod actions;
mod containers;
mod gpu;
pub(crate) mod logo;
mod rows;
mod selection;
mod tree;
mod types;

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use ratatui::prelude::Rect;
use ratatui::widgets::TableState;
use sysinfo::{
    Components, Disks, Networks, Pid, ProcessRefreshKind, RefreshKind, System, Uid, UpdateKind,
    Users,
};

use super::config::Config;
use super::highlight::HighlightMode;
use super::status::{StatusLevel, StatusMessage};
use super::view_mode::{GpuFocusPanel, ViewMode};
use crate::data::gpu::{GpuInfo, GpuPreference, GpuProcessUsage, GpuSnapshot, start_gpu_monitor};
use crate::data::{ContainerKey, ContainerRow, NetSample, ProcessRow, SortDir, SortKey};
use logo::{IconMode, LogoCache, LogoMode, LogoQuality};

pub use types::{
    ConfirmKill, GpuProcessHeaderRegion, GpuProcessSortKey, HeaderRegion, Language, SetupField,
    SystemOverviewSnapshot, SystemTab, SystemTabRegion,
};

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
    // Core system data
    pub system: System,
    pub disks: Disks,
    pub networks: Networks,
    pub components: Components,
    pub network_refresh_secs: Option<f64>,
    users: Users,
    current_user_id: Option<Uid>,

    // Process data
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
    pub tree_view: bool,
    pub rows: Vec<ProcessRow>,
    pub process_filter: String,
    pub selected_pid: Option<u32>,
    pub tree_labels: HashMap<u32, String>,
    gui_process_cache: HashMap<u32, bool>,

    // GPU data
    pub vram_enabled: bool,
    pub gpu_pref: GpuPreference,
    pub gpu_list: Vec<GpuInfo>,
    pub gpu_selected: Option<String>,
    pub gpu_processes: Vec<GpuProcessUsage>,
    pub gpu_process_order: Vec<u32>,
    gpu_rx: Option<mpsc::Receiver<GpuSnapshot>>,

    // Container data
    pub container_rows: Vec<ContainerRow>,
    pub container_selected: Option<ContainerKey>,
    pub container_pid_map: HashMap<u32, ContainerKey>,
    pub container_filter: Option<ContainerKey>,
    container_net_prev: HashMap<u64, NetSampleEntry>,
    container_net_rates: HashMap<u64, u64>,
    container_netns_cache: HashMap<ContainerKey, u64>,
    container_net_last_sample: Option<Instant>,
    network_last_refresh: Option<Instant>,

    // System info data
    pub system_overview_snapshot: Option<SystemOverviewSnapshot>,

    // Display settings
    pub icon_mode: IconMode,
    pub logo_mode: LogoMode,
    pub logo_quality: LogoQuality,
    pub logo_cache: Option<LogoCache>,
    pub language: Language,

    // View state
    pub view_mode: ViewMode,
    pub gpu_focus_panel: GpuFocusPanel,
    pub gpu_panel_expanded: bool,
    pub processes_focused: bool,
    pub processes_expanded: bool,
    pub process_filter_active: bool,
    pub highlight_mode: HighlightMode,

    // Dialogs
    pub confirm: Option<ConfirmKill>,

    // Status
    pub status: Option<StatusMessage>,

    // UI state (layout, scroll, table states)
    pub table_state: TableState,
    pub scroll: usize,
    pub process_body: Option<Rect>,
    pub process_header_regions: Vec<HeaderRegion>,
    pub gpu_process_header_regions: Vec<GpuProcessHeaderRegion>,
    pub gpu_process_body: Option<Rect>,
    pub gpu_process_scroll: usize,
    pub gpu_process_sort_key: GpuProcessSortKey,
    pub gpu_process_sort_dir: SortDir,
    pub container_table_state: TableState,
    pub container_scroll: usize,
    pub system_tab: SystemTab,
    pub system_tab_regions: Vec<SystemTabRegion>,
    pub system_update_region: Option<Rect>,
    pub show_setup: bool,
    pub show_help: bool,
    pub setup_field: SetupField,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let users = Users::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
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
            // Core system data
            system,
            disks,
            networks,
            components,
            network_refresh_secs: None,
            users,
            current_user_id,

            // Process data
            sort_key: config.sort_key,
            sort_dir: config.sort_dir,
            tree_view: false,
            rows: Vec::new(),
            process_filter: String::new(),
            selected_pid: None,
            tree_labels: HashMap::new(),
            gui_process_cache: HashMap::new(),

            // GPU data
            vram_enabled: config.vram_enabled,
            gpu_pref: config.gpu_pref,
            gpu_list: Vec::new(),
            gpu_selected: None,
            gpu_processes: Vec::new(),
            gpu_process_order: Vec::new(),
            gpu_rx,

            // Container data
            container_rows: Vec::new(),
            container_selected: None,
            container_pid_map: HashMap::new(),
            container_filter: None,
            container_net_prev: HashMap::new(),
            container_net_rates: HashMap::new(),
            container_netns_cache: HashMap::new(),
            container_net_last_sample: None,
            network_last_refresh: Some(Instant::now()),

            // System info data
            system_overview_snapshot: None,

            // Display settings
            icon_mode: config.icon_mode,
            logo_mode: config.logo_mode,
            logo_quality: config.logo_quality,
            logo_cache: None,
            language: config.language,

            // View state
            view_mode: ViewMode::default(),
            gpu_focus_panel: GpuFocusPanel::default(),
            gpu_panel_expanded: false,
            processes_focused: false,
            processes_expanded: false,
            process_filter_active: false,
            highlight_mode: HighlightMode::default(),

            // Dialogs
            confirm: None,

            // Status
            status: None,

            // UI state
            table_state: TableState::default(),
            scroll: 0,
            process_body: None,
            process_header_regions: Vec::new(),
            gpu_process_header_regions: Vec::new(),
            gpu_process_body: None,
            gpu_process_scroll: 0,
            gpu_process_sort_key: GpuProcessSortKey::Sm,
            gpu_process_sort_dir: GpuProcessSortKey::Sm.default_dir(),
            container_table_state: TableState::default(),
            container_scroll: 0,
            system_tab: SystemTab::default(),
            system_tab_regions: Vec::new(),
            system_update_region: None,
            show_setup: false,
            show_help: false,
            setup_field: SetupField::default(),
        };
        app.update_rows();
        app.poll_gpu_updates();
        app
    }

    pub fn refresh(&mut self) {
        // Use selective refresh instead of refresh_all for better performance
        let process_refresh = ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_user(UpdateKind::OnlyIfNotSet)
            .with_environ(UpdateKind::OnlyIfNotSet);
        let refresh_kind = RefreshKind::nothing().with_processes(process_refresh);
        self.system.refresh_specifics(refresh_kind);
        self.users.refresh();
        let now = Instant::now();
        self.network_refresh_secs = self
            .network_last_refresh
            .map(|previous| now.saturating_duration_since(previous).as_secs_f64())
            .filter(|value| *value > 0.0);
        self.networks.refresh(true);
        self.network_last_refresh = Some(now);
        self.disks.refresh(true);
        self.components.refresh(true);
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
            self.process_filter_active = false;
        }
        self.view_mode = mode;
    }

    pub fn toggle_gpu_focus_panel(&mut self) {
        if self.view_mode == ViewMode::GpuFocus && !self.gpu_panel_expanded {
            self.gpu_focus_panel = self.gpu_focus_panel.toggle();
        }
    }

    pub fn expand_gpu_panel(&mut self) {
        if self.view_mode == ViewMode::GpuFocus {
            self.gpu_panel_expanded = true;
        }
    }

    pub fn collapse_gpu_panel(&mut self) {
        if self.view_mode == ViewMode::GpuFocus && self.gpu_panel_expanded {
            self.gpu_panel_expanded = false;
        }
    }

    pub fn toggle_processes_focus(&mut self) {
        if self.view_mode == ViewMode::Overview && !self.processes_expanded {
            self.processes_focused = !self.processes_focused;
        }
    }

    pub fn expand_processes(&mut self) {
        if self.view_mode == ViewMode::Overview && self.processes_focused {
            self.processes_expanded = true;
        }
    }

    pub fn collapse_processes(&mut self) {
        if self.view_mode == ViewMode::Overview && self.processes_expanded {
            self.processes_expanded = false;
        }
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
        // Tree view works in Processes, Overview, and when processes are expanded
        let allowed = self.view_mode == ViewMode::Processes || self.view_mode == ViewMode::Overview;
        if !allowed {
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
            self.setup_field = SetupField::Language;
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.show_setup = false;
        }
    }

    pub fn next_setup_field(&mut self) {
        self.setup_field = self.setup_field.next();
    }

    pub fn prev_setup_field(&mut self) {
        self.setup_field = self.setup_field.prev();
    }

    pub fn toggle_setup_field(&mut self) {
        self.next_setup_value();
    }

    pub fn next_setup_value(&mut self) {
        match self.setup_field {
            SetupField::Language => self.toggle_language(),
            SetupField::IconMode => self.toggle_icon_mode(),
            SetupField::LogoMode => self.toggle_logo_mode(),
            SetupField::LogoQuality => self.next_logo_quality(),
        }
    }

    pub fn prev_setup_value(&mut self) {
        match self.setup_field {
            SetupField::Language => self.toggle_language(),
            SetupField::IconMode => self.toggle_icon_mode(),
            SetupField::LogoMode => self.toggle_logo_mode(),
            SetupField::LogoQuality => self.prev_logo_quality(),
        }
    }

    pub fn toggle_icon_mode(&mut self) {
        self.icon_mode = self.icon_mode.toggle();
        self.system_overview_snapshot = None;
        if let Err(err) = super::config::save_display_preferences(
            self.language,
            self.icon_mode,
            self.logo_mode,
            self.logo_quality,
        ) {
            self.set_status(
                StatusLevel::Warn,
                format!("Failed to save display preferences: {err}"),
            );
        }
    }

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        self.system_overview_snapshot = None;
        if let Err(err) = super::config::save_display_preferences(
            self.language,
            self.icon_mode,
            self.logo_mode,
            self.logo_quality,
        ) {
            self.set_status(
                StatusLevel::Warn,
                format!("Failed to save display preferences: {err}"),
            );
        }
    }

    pub fn toggle_logo_mode(&mut self) {
        self.logo_mode = self.logo_mode.toggle();
        if let Some(cache) = self.logo_cache.as_mut() {
            cache.rendered = None;
        }
        if let Err(err) = super::config::save_display_preferences(
            self.language,
            self.icon_mode,
            self.logo_mode,
            self.logo_quality,
        ) {
            self.set_status(
                StatusLevel::Warn,
                format!("Failed to save display preferences: {err}"),
            );
        }
    }

    pub fn next_logo_quality(&mut self) {
        self.set_logo_quality(self.logo_quality.next());
    }

    pub fn prev_logo_quality(&mut self) {
        self.set_logo_quality(self.logo_quality.prev());
    }

    fn set_logo_quality(&mut self, value: LogoQuality) {
        if self.logo_quality == value {
            return;
        }
        self.logo_quality = value;
        if let Some(cache) = self.logo_cache.as_mut() {
            cache.rendered = None;
        }
        if let Err(err) = super::config::save_display_preferences(
            self.language,
            self.icon_mode,
            self.logo_mode,
            self.logo_quality,
        ) {
            self.set_status(
                StatusLevel::Warn,
                format!("Failed to save display preferences: {err}"),
            );
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

    pub fn next_system_tab(&mut self) {
        self.system_tab = self.system_tab.next();
    }

    pub fn prev_system_tab(&mut self) {
        self.system_tab = self.system_tab.prev();
    }

    pub fn set_system_tab(&mut self, tab: SystemTab) {
        self.system_tab = tab;
    }

    pub fn system_tab_for_click(&self, column: u16, row: u16) -> Option<SystemTab> {
        self.system_tab_regions
            .iter()
            .find(|region| {
                row >= region.rect.y
                    && row < region.rect.y.saturating_add(region.rect.height)
                    && column >= region.rect.x
                    && column < region.rect.x.saturating_add(region.rect.width)
            })
            .map(|region| region.tab)
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
