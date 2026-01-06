use std::cmp::Ordering;

use super::ProcessRow;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggle(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "asc" => Some(SortDir::Asc),
            "desc" => Some(SortDir::Desc),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Pid,
    User,
    Cpu,
    Mem,
    Uptime,
    Status,
    Name,
}

impl SortKey {
    pub fn label(self) -> &'static str {
        match self {
            SortKey::Pid => "pid",
            SortKey::User => "user",
            SortKey::Cpu => "cpu",
            SortKey::Mem => "mem",
            SortKey::Uptime => "uptime",
            SortKey::Status => "stat",
            SortKey::Name => "name",
        }
    }

    pub fn default_dir(self) -> SortDir {
        match self {
            SortKey::Cpu | SortKey::Mem | SortKey::Uptime => SortDir::Desc,
            SortKey::Pid | SortKey::User | SortKey::Status | SortKey::Name => SortDir::Asc,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "pid" => Some(SortKey::Pid),
            "user" => Some(SortKey::User),
            "cpu" => Some(SortKey::Cpu),
            "mem" => Some(SortKey::Mem),
            "up" | "uptime" => Some(SortKey::Uptime),
            "stat" | "status" => Some(SortKey::Status),
            "name" => Some(SortKey::Name),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            SortKey::Pid => SortKey::User,
            SortKey::User => SortKey::Cpu,
            SortKey::Cpu => SortKey::Mem,
            SortKey::Mem => SortKey::Uptime,
            SortKey::Uptime => SortKey::Status,
            SortKey::Status => SortKey::Name,
            SortKey::Name => SortKey::Pid,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SortKey::Pid => SortKey::Name,
            SortKey::User => SortKey::Pid,
            SortKey::Cpu => SortKey::User,
            SortKey::Mem => SortKey::Cpu,
            SortKey::Uptime => SortKey::Mem,
            SortKey::Status => SortKey::Uptime,
            SortKey::Name => SortKey::Status,
        }
    }
}

pub fn sort_process_rows(rows: &mut [ProcessRow], sort_key: SortKey, sort_dir: SortDir) {
    rows.sort_by(|a, b| {
        let ordering = match sort_key {
            SortKey::Pid => a.pid.cmp(&b.pid),
            SortKey::User => match (&a.user, &b.user) {
                (Some(a), Some(b)) => a.cmp(b),
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (None, None) => Ordering::Equal,
            },
            SortKey::Cpu => a.cpu.partial_cmp(&b.cpu).unwrap_or(Ordering::Equal),
            SortKey::Mem => a.mem_bytes.cmp(&b.mem_bytes),
            SortKey::Uptime => a.uptime_secs.cmp(&b.uptime_secs),
            SortKey::Status => a.status.cmp(&b.status),
            SortKey::Name => a.name.cmp(&b.name),
        };

        let ordering = match sort_dir {
            SortDir::Asc => ordering,
            SortDir::Desc => ordering.reverse(),
        };

        ordering.then_with(|| a.pid.cmp(&b.pid))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_process_rows_by_cpu_desc() {
        let mut rows = vec![
            ProcessRow {
                pid: 2,
                user: None,
                name: "b".to_string(),
                cpu: 20.0,
                mem_bytes: 200,
                status: "Sleep".to_string(),
                start_time: 0,
                uptime_secs: 20,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
            ProcessRow {
                pid: 1,
                user: None,
                name: "a".to_string(),
                cpu: 20.0,
                mem_bytes: 100,
                status: "Run".to_string(),
                start_time: 0,
                uptime_secs: 30,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
            ProcessRow {
                pid: 3,
                user: None,
                name: "c".to_string(),
                cpu: 10.0,
                mem_bytes: 300,
                status: "Run".to_string(),
                start_time: 0,
                uptime_secs: 10,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
        ];

        sort_process_rows(&mut rows, SortKey::Cpu, SortDir::Desc);

        assert_eq!(rows[0].pid, 1);
        assert_eq!(rows[1].pid, 2);
        assert_eq!(rows[2].pid, 3);
    }

    #[test]
    fn sort_process_rows_by_user_asc() {
        let mut rows = vec![
            ProcessRow {
                pid: 1,
                user: Some("bob".to_string()),
                name: "b".to_string(),
                cpu: 20.0,
                mem_bytes: 200,
                status: "Sleep".to_string(),
                start_time: 0,
                uptime_secs: 20,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
            ProcessRow {
                pid: 2,
                user: None,
                name: "a".to_string(),
                cpu: 20.0,
                mem_bytes: 100,
                status: "Run".to_string(),
                start_time: 0,
                uptime_secs: 30,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
            ProcessRow {
                pid: 3,
                user: Some("alice".to_string()),
                name: "c".to_string(),
                cpu: 10.0,
                mem_bytes: 300,
                status: "Run".to_string(),
                start_time: 0,
                uptime_secs: 10,
                is_current_user: false,
                is_non_root: false,
                is_gui: false,
                gpu_sm_pct: None,
                gpu_mem_pct: None,
                gpu_enc_pct: None,
                gpu_dec_pct: None,
                gpu_fb_bytes: None,
                gpu_kind: None,
            },
        ];

        sort_process_rows(&mut rows, SortKey::User, SortDir::Asc);

        assert_eq!(rows[0].user.as_deref(), Some("alice"));
        assert_eq!(rows[1].user.as_deref(), Some("bob"));
        assert_eq!(rows[2].user.as_deref(), None);
    }
}
