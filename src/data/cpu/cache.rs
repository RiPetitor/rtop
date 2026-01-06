use std::fs;
use std::path::Path;

/// CPU cache sizes
#[derive(Debug, Clone, Default)]
pub struct CpuCaches {
    /// L1 Data cache size in KB
    pub l1d: u32,
    /// L1 Instruction cache size in KB
    pub l1i: u32,
    /// L2 cache size in KB
    pub l2: u32,
    /// L3 cache size in KB (may be shared between cores)
    pub l3: u32,
    /// Number of L1d caches (usually = number of cores)
    pub l1d_count: u32,
    /// Number of L2 caches
    pub l2_count: u32,
    /// Number of L3 caches (usually 1 per CCD or socket)
    pub l3_count: u32,
}

impl CpuCaches {
    pub fn read() -> Self {
        let mut caches = Self::default();
        let cpu_path = Path::new("/sys/devices/system/cpu/cpu0/cache");

        if !cpu_path.exists() {
            return caches;
        }

        // Read cache info for cpu0
        for index in 0..10 {
            let index_path = cpu_path.join(format!("index{}", index));
            if !index_path.exists() {
                break;
            }

            let level = read_file_u32(&index_path.join("level")).unwrap_or(0);
            let cache_type = read_file_string(&index_path.join("type")).unwrap_or_default();
            let size_str = read_file_string(&index_path.join("size")).unwrap_or_default();
            let size_kb = parse_cache_size(&size_str);

            match (level, cache_type.as_str()) {
                (1, "Data") => caches.l1d = size_kb,
                (1, "Instruction") => caches.l1i = size_kb,
                (2, "Unified") => caches.l2 = size_kb,
                (3, "Unified") => caches.l3 = size_kb,
                _ => {}
            }
        }

        // Count caches across all CPUs
        caches.count_caches();

        caches
    }

    fn count_caches(&mut self) {
        let cpu_base = Path::new("/sys/devices/system/cpu");

        for cpu_id in 0..1024 {
            let cpu_path = cpu_base.join(format!("cpu{}", cpu_id));
            if !cpu_path.exists() {
                break;
            }

            let cache_path = cpu_path.join("cache");
            if !cache_path.exists() {
                continue;
            }

            // Check which cache indices exist for this CPU
            for index in 0..10 {
                let index_path = cache_path.join(format!("index{}", index));
                if !index_path.exists() {
                    break;
                }

                let level = read_file_u32(&index_path.join("level")).unwrap_or(0);
                let cache_type = read_file_string(&index_path.join("type")).unwrap_or_default();
                let shared_cpus =
                    read_file_string(&index_path.join("shared_cpu_list")).unwrap_or_default();

                // Only count if this CPU is the first in the shared list
                let is_first = shared_cpus
                    .split(',')
                    .next()
                    .and_then(|s| s.split('-').next())
                    .and_then(|s| s.parse::<u32>().ok())
                    .map(|first| first == cpu_id)
                    .unwrap_or(true);

                if is_first {
                    match (level, cache_type.as_str()) {
                        (1, "Data") => self.l1d_count += 1,
                        (2, "Unified") => self.l2_count += 1,
                        (3, "Unified") => self.l3_count += 1,
                        _ => {}
                    }
                }
            }
        }
    }

    /// Format L1 cache for display (e.g., "32 KB x8" or "32+32 KB x8")
    pub fn format_l1(&self) -> String {
        if self.l1d == 0 && self.l1i == 0 {
            return "N/A".to_string();
        }

        let count = if self.l1d_count > 0 {
            self.l1d_count
        } else {
            1
        };

        if self.l1d == self.l1i {
            format!("{}+{} KB x{}", self.l1d, self.l1i, count)
        } else if self.l1i == 0 {
            format!("{} KB x{}", self.l1d, count)
        } else {
            format!("{}+{} KB x{}", self.l1d, self.l1i, count)
        }
    }

    /// Format L2 cache for display (e.g., "512 KB x8")
    pub fn format_l2(&self) -> String {
        if self.l2 == 0 {
            return "N/A".to_string();
        }

        let count = if self.l2_count > 0 { self.l2_count } else { 1 };

        if self.l2 >= 1024 {
            format!("{} MB x{}", self.l2 / 1024, count)
        } else {
            format!("{} KB x{}", self.l2, count)
        }
    }

    /// Format L3 cache for display (e.g., "32 MB")
    pub fn format_l3(&self) -> String {
        if self.l3 == 0 {
            return "N/A".to_string();
        }

        let count = if self.l3_count > 0 { self.l3_count } else { 1 };

        if self.l3 >= 1024 {
            if count > 1 {
                format!("{} MB x{}", self.l3 / 1024, count)
            } else {
                format!("{} MB", self.l3 / 1024)
            }
        } else {
            format!("{} KB", self.l3)
        }
    }
}

fn read_file_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_file_u32(path: &Path) -> Option<u32> {
    read_file_string(path)?.parse().ok()
}

fn parse_cache_size(size_str: &str) -> u32 {
    let size_str = size_str.trim();
    if size_str.ends_with('K') {
        size_str.trim_end_matches('K').parse().unwrap_or(0)
    } else if size_str.ends_with('M') {
        size_str
            .trim_end_matches('M')
            .parse::<u32>()
            .unwrap_or(0)
            .saturating_mul(1024)
    } else {
        size_str.parse().unwrap_or(0)
    }
}
