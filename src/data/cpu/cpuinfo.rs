use std::fs;

const CPUINFO_PATH: &str = "/proc/cpuinfo";

/// Detailed CPU information parsed from /proc/cpuinfo
#[derive(Debug, Clone, Default)]
pub struct CpuDetails {
    pub vendor_id: String,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub model_name: String,
    pub microcode: String,
    pub flags: Vec<String>,
    pub bugs: Vec<String>,
    pub bogomips: f64,
    pub address_sizes: String,
}

impl CpuDetails {
    pub fn read() -> Self {
        let content = match fs::read_to_string(CPUINFO_PATH) {
            Ok(content) => content,
            Err(_) => return Self::default(),
        };

        let mut details = Self::default();
        let mut flags_found = false;

        for line in content.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "vendor_id" if details.vendor_id.is_empty() => {
                    details.vendor_id = value.to_string();
                }
                "cpu family" if details.family == 0 => {
                    details.family = value.parse().unwrap_or(0);
                }
                "model" if details.model == 0 && key == "model" => {
                    details.model = value.parse().unwrap_or(0);
                }
                "stepping" if details.stepping == 0 => {
                    details.stepping = value.parse().unwrap_or(0);
                }
                "model name" if details.model_name.is_empty() => {
                    details.model_name = value.to_string();
                }
                "microcode" if details.microcode.is_empty() => {
                    details.microcode = value.to_string();
                }
                "flags" if !flags_found => {
                    details.flags = value.split_whitespace().map(String::from).collect();
                    flags_found = true;
                }
                "bugs" if details.bugs.is_empty() => {
                    details.bugs = value.split_whitespace().map(String::from).collect();
                }
                "bogomips" if details.bogomips == 0.0 => {
                    details.bogomips = value.parse().unwrap_or(0.0);
                }
                "address sizes" if details.address_sizes.is_empty() => {
                    details.address_sizes = value.to_string();
                }
                _ => {}
            }

            // Stop after first CPU block since all CPUs have same info
            if flags_found && !details.bugs.is_empty() {
                break;
            }
        }

        details
    }

    /// Get formatted family/model/stepping string
    pub fn family_model_stepping(&self) -> String {
        format!(
            "Family {:X}h, Model {:X}h, Stepping {}",
            self.family, self.model, self.stepping
        )
    }

    /// Get extended family (for AMD: family + ext_family)
    pub fn extended_family(&self) -> u32 {
        if self.family == 0x0F {
            // Extended family is in bits 27:20 of CPUID, but we calculate from family
            self.family
        } else {
            self.family
        }
    }

    /// Get extended model (for AMD: (ext_model << 4) + model)
    pub fn extended_model(&self) -> u32 {
        self.model
    }

    /// Check if specific instruction set is supported
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }

    /// Get key instruction sets for display
    pub fn key_instructions(&self) -> Vec<&'static str> {
        let mut instructions = Vec::new();

        // x86-64 levels
        if self.has_flag("lm") {
            instructions.push("x86-64");
        }

        // SSE family
        if self.has_flag("sse") {
            instructions.push("SSE");
        }
        if self.has_flag("sse2") {
            instructions.push("SSE2");
        }
        if self.has_flag("sse4_1") {
            instructions.push("SSE4.1");
        }
        if self.has_flag("sse4_2") {
            instructions.push("SSE4.2");
        }

        // AVX family
        if self.has_flag("avx") {
            instructions.push("AVX");
        }
        if self.has_flag("avx2") {
            instructions.push("AVX2");
        }
        if self.has_flag("avx512f") {
            instructions.push("AVX-512");
        }

        // Other important
        if self.has_flag("aes") {
            instructions.push("AES");
        }
        if self.has_flag("fma") || self.has_flag("fma3") {
            instructions.push("FMA3");
        }
        if self.has_flag("sha_ni") {
            instructions.push("SHA");
        }

        instructions
    }

    /// Check if running as root
    pub fn is_root() -> bool {
        std::fs::metadata("/root")
            .map(|m| m.is_dir())
            .unwrap_or(false)
            && std::fs::read_dir("/root").is_ok()
    }

    /// Bus speed in MHz (100 MHz for modern CPUs)
    pub fn bus_speed_mhz() -> u32 {
        100
    }

    /// Calculate multiplier from frequency
    pub fn multiplier_from_freq(freq_mhz: u64) -> f64 {
        freq_mhz as f64 / Self::bus_speed_mhz() as f64
    }
}
