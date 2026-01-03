#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
    Crio,
    Kubernetes,
}

impl ContainerRuntime {
    pub fn label(self) -> &'static str {
        match self {
            ContainerRuntime::Docker => "docker",
            ContainerRuntime::Podman => "podman",
            ContainerRuntime::Containerd => "containerd",
            ContainerRuntime::Crio => "crio",
            ContainerRuntime::Kubernetes => "k8s",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContainerKey {
    pub runtime: ContainerRuntime,
    pub id: String,
}

impl ContainerKey {
    pub fn label(&self) -> String {
        format!("{}:{}", self.runtime.label(), short_id(&self.id))
    }
}

#[derive(Clone, Debug)]
pub struct ContainerRow {
    pub key: ContainerKey,
    pub label: String,
    pub cpu: f32,
    pub mem_bytes: u64,
    pub proc_count: usize,
    pub net_bytes_per_sec: Option<u64>,
}

impl ContainerRow {
    pub fn new(
        key: ContainerKey,
        cpu: f32,
        mem_bytes: u64,
        proc_count: usize,
        net_bytes_per_sec: Option<u64>,
    ) -> Self {
        let label = key.label();
        Self {
            key,
            label,
            cpu,
            mem_bytes,
            proc_count,
            net_bytes_per_sec,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NetSample {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

fn short_id(value: &str) -> String {
    value.chars().take(12).collect()
}
