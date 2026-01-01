use std::sync::mpsc;

use ratatui::widgets::TableState;
use sysinfo::{Pid, Signal, System};

use super::config::Config;
use super::status::{StatusLevel, StatusMessage};
use crate::data::gpu::{GpuInfo, GpuPreference, GpuSnapshot, default_gpu_index, start_gpu_monitor};
use crate::data::{ProcessRow, SortDir, SortKey, sort_process_rows};

pub struct ConfirmKill {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub start_time: u64,
}

pub struct App {
    pub system: System,
    pub sort_key: SortKey,
    pub sort_dir: SortDir,
    pub rows: Vec<ProcessRow>,
    pub table_state: TableState,
    pub selected_pid: Option<u32>,
    pub scroll: usize,
    pub confirm: Option<ConfirmKill>,
    pub vram_enabled: bool,
    pub gpu_pref: GpuPreference,
    pub gpu_list: Vec<GpuInfo>,
    pub gpu_selected: Option<String>,
    gpu_rx: Option<mpsc::Receiver<GpuSnapshot>>,
    pub status: Option<StatusMessage>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let gpu_rx = if config.vram_enabled {
            Some(start_gpu_monitor())
        } else {
            None
        };
        let mut app = Self {
            system,
            sort_key: config.sort_key,
            sort_dir: config.sort_dir,
            rows: Vec::new(),
            table_state: TableState::default(),
            selected_pid: None,
            scroll: 0,
            confirm: None,
            vram_enabled: config.vram_enabled,
            gpu_pref: config.gpu_pref,
            gpu_list: Vec::new(),
            gpu_selected: None,
            gpu_rx,
            status: None,
        };
        app.update_rows();
        app.poll_gpu_updates();
        app
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.update_rows();
    }

    pub fn tick(&mut self) {
        self.poll_gpu_updates();
        self.clear_expired_status();
    }

    pub fn set_sort_key(&mut self, key: SortKey) {
        self.sort_key = key;
        self.sort_dir = key.default_dir();
        self.update_rows();
    }

    pub fn toggle_sort_dir(&mut self) {
        self.sort_dir = self.sort_dir.toggle();
        self.update_rows();
    }

    pub fn update_rows(&mut self) {
        let mut rows = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessRow {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu: process.cpu_usage(),
                mem_bytes: process.memory(),
                status: format!("{:?}", process.status()),
                start_time: process.start_time(),
                uptime_secs: process.run_time(),
            })
            .collect::<Vec<_>>();

        sort_process_rows(&mut rows, self.sort_key, self.sort_dir);
        self.rows = rows;
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

    pub fn open_confirm(&mut self) {
        if let Some(row) = self.selected_row() {
            self.confirm = Some(ConfirmKill {
                pid: row.pid,
                name: row.name.clone(),
                cpu: row.cpu,
                mem_bytes: row.mem_bytes,
                status: row.status.clone(),
                start_time: row.start_time,
            });
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
    }

    pub fn confirm_kill(&mut self) {
        if let Some(confirm) = self.confirm.take() {
            let pid = Pid::from_u32(confirm.pid);
            self.system.refresh_process(pid);
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

        if let Some(selected) = self.gpu_selected.as_ref() {
            if self.gpu_list.iter().any(|gpu| &gpu.id == selected) {
                return;
            }
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

    fn clear_expired_status(&mut self) {
        if let Some(status) = self.status.as_ref() {
            if status.is_expired() {
                self.status = None;
            }
        }
    }

    /// Apply GPU snapshot from event system
    pub fn apply_gpu_snapshot(&mut self, snapshot: crate::data::gpu::GpuSnapshot) {
        self.update_gpu_list(snapshot.gpus);
    }
}
