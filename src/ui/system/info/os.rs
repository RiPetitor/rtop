use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::app::Language;

use super::icons::ICON_IMMUTABLE;

#[derive(Default, Clone)]
pub struct OsRelease {
    pub name: Option<String>,
    pub pretty_name: Option<String>,
    pub id: Option<String>,
    pub version: Option<String>,
    pub version_id: Option<String>,
    pub variant: Option<String>,
    pub variant_id: Option<String>,
    pub image_id: Option<String>,
    pub build_id: Option<String>,
}

pub fn os_release() -> OsRelease {
    static CACHE: OnceLock<OsRelease> = OnceLock::new();
    CACHE.get_or_init(load_os_release).clone()
}

fn load_os_release() -> OsRelease {
    let content = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    parse_os_release(&content)
}

fn parse_os_release(content: &str) -> OsRelease {
    let mut info = OsRelease::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key {
            "NAME" => info.name = Some(value),
            "PRETTY_NAME" => info.pretty_name = Some(value),
            "ID" => info.id = Some(value),
            "VERSION" => info.version = Some(value),
            "VERSION_ID" => info.version_id = Some(value),
            "VARIANT" => info.variant = Some(value),
            "VARIANT_ID" => info.variant_id = Some(value),
            "IMAGE_ID" => info.image_id = Some(value),
            "BUILD_ID" => info.build_id = Some(value),
            _ => {}
        }
    }
    info
}

pub fn distro_variant_line(info: &OsRelease) -> Option<String> {
    let id = info
        .image_id
        .as_ref()
        .or(info.id.as_ref())
        .or(info.name.as_ref())?;
    let mut line = id.clone();
    if let Some(variant) = info.variant_id.as_ref().or(info.variant.as_ref()) {
        line.push(':');
        line.push_str(variant);
    } else if let Some(version) = info.version_id.as_ref() {
        line.push(':');
        line.push_str(version);
    }
    if is_immutable_os() {
        line.push(' ');
        line.push_str(ICON_IMMUTABLE);
    }
    Some(line)
}

pub fn is_immutable_os() -> bool {
    Path::new("/run/ostree-booted").exists() || Path::new("/sysroot/ostree").exists()
}

pub fn format_uptime_long(uptime_secs: u64, language: Language) -> String {
    let mut remaining = uptime_secs;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let minutes = remaining / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!(
            "{days} {}",
            pluralize(language, days, "day", "days", "день", "дня", "дней")
        ));
    }
    if hours > 0 {
        parts.push(format!(
            "{hours} {}",
            pluralize(language, hours, "hour", "hours", "час", "часа", "часов")
        ));
    }
    if minutes > 0 || parts.is_empty() {
        parts.push(format!(
            "{minutes} {}",
            pluralize(
                language,
                minutes,
                "min",
                "mins",
                "минута",
                "минуты",
                "минут"
            )
        ));
    }
    parts.join(", ")
}

pub fn pluralize<'a>(
    language: Language,
    value: u64,
    en_one: &'a str,
    en_many: &'a str,
    ru_one: &'a str,
    ru_few: &'a str,
    ru_many: &'a str,
) -> &'a str {
    match language {
        Language::English => {
            if value == 1 {
                en_one
            } else {
                en_many
            }
        }
        Language::Russian => {
            let mod10 = value % 10;
            let mod100 = value % 100;
            if mod10 == 1 && mod100 != 11 {
                ru_one
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                ru_few
            } else {
                ru_many
            }
        }
    }
}
