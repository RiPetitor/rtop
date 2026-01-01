pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub start_time: u64,
    pub uptime_secs: u64,
}
