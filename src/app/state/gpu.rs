use super::App;
use crate::data::gpu::{GpuInfo, default_gpu_index};

impl App {
    pub fn poll_gpu_updates(&mut self) {
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

    pub(super) fn update_gpu_list(&mut self, mut gpus: Vec<GpuInfo>) {
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
}
