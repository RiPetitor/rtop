use super::App;

impl App {
    pub(super) fn sync_selection(&mut self) {
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

    pub fn selected_row(&self) -> Option<&crate::data::ProcessRow> {
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
}
