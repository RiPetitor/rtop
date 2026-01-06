/// CPU codename information from database
#[derive(Debug, Clone)]
pub struct CpuCodename {
    /// Marketing codename (e.g., "Raphael", "Raptor Lake")
    pub codename: &'static str,
    /// Socket/Package (e.g., "AM5", "LGA1700")
    pub package: &'static str,
    /// Manufacturing process (e.g., "5 nm", "Intel 7")
    pub technology: &'static str,
}

/// Lookup CPU codename by vendor, family, and model
pub fn lookup_cpu_codename(vendor: &str, family: u32, model: u32) -> Option<CpuCodename> {
    match vendor {
        "AuthenticAMD" | "AMD" => lookup_amd(family, model),
        "GenuineIntel" | "Intel" => lookup_intel(family, model),
        _ => None,
    }
}

fn lookup_amd(family: u32, model: u32) -> Option<CpuCodename> {
    // AMD Family 25h (Zen 3/4)
    if family == 25 {
        return match model {
            // Zen 4 - Raphael (Desktop AM5)
            97 => Some(CpuCodename {
                codename: "Raphael",
                package: "AM5",
                technology: "5 nm",
            }),
            // Zen 4 - Phoenix (Mobile)
            117 => Some(CpuCodename {
                codename: "Phoenix",
                package: "FP7/FP8",
                technology: "4 nm",
            }),
            // Zen 4c - Bergamo (Server)
            160 => Some(CpuCodename {
                codename: "Bergamo",
                package: "SP5",
                technology: "5 nm",
            }),
            // Zen 4 - Genoa (Server)
            17 => Some(CpuCodename {
                codename: "Genoa",
                package: "SP5",
                technology: "5 nm",
            }),
            // Zen 3 - Vermeer (Desktop AM4)
            33 => Some(CpuCodename {
                codename: "Vermeer",
                package: "AM4",
                technology: "7 nm",
            }),
            // Zen 3 - Cezanne (APU)
            80 => Some(CpuCodename {
                codename: "Cezanne",
                package: "AM4/FP6",
                technology: "7 nm",
            }),
            // Zen 3+ - Rembrandt (Mobile)
            68 => Some(CpuCodename {
                codename: "Rembrandt",
                package: "FP7",
                technology: "6 nm",
            }),
            _ => Some(CpuCodename {
                codename: "Zen 3/4",
                package: "Unknown",
                technology: "7-5 nm",
            }),
        };
    }

    // AMD Family 26h (Zen 5)
    if family == 26 {
        return match model {
            // Zen 5 - Granite Ridge (Desktop AM5)
            32..=63 => Some(CpuCodename {
                codename: "Granite Ridge",
                package: "AM5",
                technology: "4 nm",
            }),
            // Zen 5 - Strix Point (Mobile)
            64..=95 => Some(CpuCodename {
                codename: "Strix Point",
                package: "FP8",
                technology: "4 nm",
            }),
            // Zen 5 - Turin (Server)
            0..=31 => Some(CpuCodename {
                codename: "Turin",
                package: "SP5",
                technology: "4 nm",
            }),
            _ => Some(CpuCodename {
                codename: "Zen 5",
                package: "Unknown",
                technology: "4 nm",
            }),
        };
    }

    // AMD Family 23h (Zen/Zen+/Zen 2)
    if family == 23 {
        return match model {
            // Zen - Summit Ridge
            1 => Some(CpuCodename {
                codename: "Summit Ridge",
                package: "AM4",
                technology: "14 nm",
            }),
            // Zen+ - Pinnacle Ridge
            8 => Some(CpuCodename {
                codename: "Pinnacle Ridge",
                package: "AM4",
                technology: "12 nm",
            }),
            // Zen 2 - Matisse
            113 => Some(CpuCodename {
                codename: "Matisse",
                package: "AM4",
                technology: "7 nm",
            }),
            // Zen 2 - Renoir (APU)
            96 => Some(CpuCodename {
                codename: "Renoir",
                package: "AM4/FP6",
                technology: "7 nm",
            }),
            _ => Some(CpuCodename {
                codename: "Zen/Zen+/Zen 2",
                package: "AM4",
                technology: "14-7 nm",
            }),
        };
    }

    None
}

fn lookup_intel(family: u32, model: u32) -> Option<CpuCodename> {
    // Intel Family 6 (most modern Intel CPUs)
    if family == 6 {
        return match model {
            // Raptor Lake (13th/14th Gen)
            183 | 191 => Some(CpuCodename {
                codename: "Raptor Lake",
                package: "LGA1700",
                technology: "Intel 7",
            }),
            // Alder Lake (12th Gen)
            151 | 154 => Some(CpuCodename {
                codename: "Alder Lake",
                package: "LGA1700",
                technology: "Intel 7",
            }),
            // Rocket Lake (11th Gen)
            167 => Some(CpuCodename {
                codename: "Rocket Lake",
                package: "LGA1200",
                technology: "14 nm",
            }),
            // Comet Lake (10th Gen)
            165 => Some(CpuCodename {
                codename: "Comet Lake",
                package: "LGA1200",
                technology: "14 nm",
            }),
            // Tiger Lake (Mobile 11th Gen)
            140 | 141 => Some(CpuCodename {
                codename: "Tiger Lake",
                package: "BGA",
                technology: "10 nm SF",
            }),
            // Ice Lake (Mobile 10th Gen)
            126 => Some(CpuCodename {
                codename: "Ice Lake",
                package: "BGA",
                technology: "10 nm",
            }),
            // Coffee Lake (8th/9th Gen)
            158 | 142 => Some(CpuCodename {
                codename: "Coffee Lake",
                package: "LGA1151",
                technology: "14 nm",
            }),
            // Skylake
            94 | 78 => Some(CpuCodename {
                codename: "Skylake",
                package: "LGA1151",
                technology: "14 nm",
            }),
            // Arrow Lake (Core Ultra 200)
            198 => Some(CpuCodename {
                codename: "Arrow Lake",
                package: "LGA1851",
                technology: "Intel 20A",
            }),
            // Meteor Lake (Core Ultra)
            170 => Some(CpuCodename {
                codename: "Meteor Lake",
                package: "BGA",
                technology: "Intel 4",
            }),
            // Sapphire Rapids (Server)
            143 => Some(CpuCodename {
                codename: "Sapphire Rapids",
                package: "LGA4677",
                technology: "Intel 7",
            }),
            _ => None,
        };
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amd_raphael() {
        let info = lookup_cpu_codename("AuthenticAMD", 25, 97);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.codename, "Raphael");
        assert_eq!(info.package, "AM5");
        assert_eq!(info.technology, "5 nm");
    }

    #[test]
    fn test_intel_raptor_lake() {
        let info = lookup_cpu_codename("GenuineIntel", 6, 183);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.codename, "Raptor Lake");
    }
}
