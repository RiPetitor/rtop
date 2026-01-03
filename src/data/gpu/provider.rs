use std::time::Duration;

use super::types::GpuInfo;

/// Trait for GPU information providers
/// Each provider implements a different method of discovering GPUs
pub trait GpuProvider: Send + Sync {
    /// Human-readable name of this provider
    fn name(&self) -> &'static str;

    /// Priority for merging (higher = preferred source)
    fn priority(&self) -> u8;

    /// Probe for GPUs, optionally skipping NVIDIA GPUs
    fn probe(&self, skip_nvidia: bool) -> Vec<GpuInfo>;

    /// Timeout for this provider's operations
    fn timeout(&self) -> Duration {
        Duration::from_millis(800)
    }
}

/// NVIDIA provider using nvidia-smi
pub struct NvidiaProvider;

impl GpuProvider for NvidiaProvider {
    fn name(&self) -> &'static str {
        "nvidia-smi"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority - most accurate for NVIDIA
    }

    fn probe(&self, _skip_nvidia: bool) -> Vec<GpuInfo> {
        super::nvidia::probe_nvidia_gpus(self.timeout())
    }
}

/// lspci provider for PCI device enumeration
pub struct LspciProvider;

impl GpuProvider for LspciProvider {
    fn name(&self) -> &'static str {
        "lspci"
    }

    fn priority(&self) -> u8 {
        50 // Medium priority
    }

    fn probe(&self, skip_nvidia: bool) -> Vec<GpuInfo> {
        super::lspci::probe_lspci_gpus(self.timeout(), skip_nvidia)
    }
}

/// sysfs provider for Linux DRM subsystem
pub struct SysfsProvider;

impl GpuProvider for SysfsProvider {
    fn name(&self) -> &'static str {
        "sysfs"
    }

    fn priority(&self) -> u8 {
        25 // Lower priority - fallback
    }

    fn probe(&self, skip_nvidia: bool) -> Vec<GpuInfo> {
        super::sysfs::probe_sysfs_gpus(skip_nvidia)
    }
}

/// Collection of GPU providers
pub struct GpuProviderRegistry {
    providers: Vec<Box<dyn GpuProvider>>,
}

impl GpuProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Create registry with default providers
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(NvidiaProvider));
        registry.register(Box::new(LspciProvider));
        registry.register(Box::new(SysfsProvider));
        registry
    }

    pub fn register(&mut self, provider: Box<dyn GpuProvider>) {
        self.providers.push(provider);
    }

    /// Probe all providers and return merged results
    pub fn probe_all(&self) -> Vec<GpuInfo> {
        // Collect results from all providers, sorted by priority
        let mut sorted_providers: Vec<_> = self.providers.iter().collect();
        sorted_providers.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        let mut cached_nvidia = None;
        let mut has_nvidia = false;
        for provider in &sorted_providers {
            if provider.name() == "nvidia-smi" {
                let gpus = provider.probe(false);
                has_nvidia = !gpus.is_empty();
                cached_nvidia = Some(gpus);
                break;
            }
        }

        let mut all_gpus: Vec<Vec<GpuInfo>> = Vec::new();
        for provider in sorted_providers {
            let gpus = if provider.name() == "nvidia-smi" {
                cached_nvidia
                    .take()
                    .unwrap_or_else(|| provider.probe(false))
            } else {
                provider.probe(has_nvidia)
            };
            all_gpus.push(gpus);
        }

        // Merge results
        super::merge_gpu_lists_multi(all_gpus)
    }
}

impl Default for GpuProviderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
