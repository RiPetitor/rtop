mod cache;
mod cpuinfo;
mod database;

pub use cache::CpuCaches;
pub use cpuinfo::CpuDetails;
pub use database::{CpuCodename, lookup_cpu_codename};

use std::sync::OnceLock;

static CPU_INFO: OnceLock<CpuDetails> = OnceLock::new();
static CPU_CACHES: OnceLock<CpuCaches> = OnceLock::new();

/// Get cached CPU details (parsed once from /proc/cpuinfo)
pub fn cpu_details() -> &'static CpuDetails {
    CPU_INFO.get_or_init(CpuDetails::read)
}

/// Get cached CPU cache sizes (parsed once from /sys)
pub fn cpu_caches() -> &'static CpuCaches {
    CPU_CACHES.get_or_init(CpuCaches::read)
}
