use sysinfo::{Pid, ProcessesToUpdate, Signal};

use super::{App, ConfirmKill, StatusLevel};

impl App {
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
}
