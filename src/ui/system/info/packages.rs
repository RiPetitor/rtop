use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crate::utils::run_command_with_timeout;

pub fn package_summary() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(package_summary_inner).clone()
}

fn package_summary_inner() -> Option<String> {
    let mut parts = Vec::new();
    let timeout = Duration::from_secs(2);

    // RPM-based (Fedora, RHEL, openSUSE) - exclude gpg-pubkey packages
    if let Some(count) = count_rpm(timeout)
        && count > 0
    {
        parts.push(format!("{count} (rpm)"));
    }
    // Debian-based (Debian, Ubuntu)
    if let Some(count) =
        count_command_lines("dpkg-query", &["-f", "${binary:Package}\\n", "-W"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (dpkg)"));
    }
    // Arch-based
    if let Some(count) = count_command_lines("pacman", &["-Qq"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (pacman)"));
    }
    // Gentoo
    if let Some(count) = count_portage()
        && count > 0
    {
        parts.push(format!("{count} (portage)"));
    }
    // Void Linux
    if let Some(count) = count_command_lines("xbps-query", &["-l"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (xbps)"));
    }
    // Alpine Linux
    if let Some(count) = count_apk(timeout)
        && count > 0
    {
        parts.push(format!("{count} (apk)"));
    }
    // Solus
    if let Some(count) = count_command_lines("eopkg", &["li"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (eopkg)"));
    }
    // NixOS / Nix
    if let Some(count) = count_nix(timeout)
        && count > 0
    {
        parts.push(format!("{count} (nix)"));
    }
    // Flatpak (all packages including runtimes)
    if let Some(count) = count_flatpak(timeout)
        && count > 0
    {
        parts.push(format!("{count} (flatpak)"));
    }
    // Snap
    if let Some(count) = count_command_lines("snap", &["list"], timeout) {
        let count = count.saturating_sub(1); // Skip header line
        if count > 0 {
            parts.push(format!("{count} (snap)"));
        }
    }
    // Homebrew (macOS/Linux)
    if let Some(count) = count_command_lines("brew", &["list", "--formula"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (brew)"));
    }
    if let Some(count) = count_command_lines("brew", &["list", "--cask"], timeout)
        && count > 0
    {
        parts.push(format!("{count} (brew-cask)"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn count_rpm(timeout: Duration) -> Option<usize> {
    if !command_exists("rpm") {
        return None;
    }
    let output = run_command_with_timeout("rpm", &["-qa"], timeout)?;
    // Exclude gpg-pubkey packages (like fastfetch does)
    let count = output
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("gpg-pubkey")
        })
        .count();
    Some(count)
}

fn count_portage() -> Option<usize> {
    // Count installed packages in /var/db/pkg
    let pkg_dir = Path::new("/var/db/pkg");
    if !pkg_dir.exists() {
        return None;
    }
    let mut count = 0;
    if let Ok(categories) = fs::read_dir(pkg_dir) {
        for cat_entry in categories.flatten() {
            if let Ok(packages) = fs::read_dir(cat_entry.path()) {
                count += packages.count();
            }
        }
    }
    if count > 0 { Some(count) } else { None }
}

fn count_apk(timeout: Duration) -> Option<usize> {
    if !command_exists("apk") {
        return None;
    }
    let output = run_command_with_timeout("apk", &["info"], timeout)?;
    let count = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    Some(count)
}

fn count_nix(timeout: Duration) -> Option<usize> {
    if !command_exists("nix-store") {
        return None;
    }
    // Count packages in system profile or user profile
    let output = run_command_with_timeout("nix-store", &["-qR", "/run/current-system/sw"], timeout)
        .or_else(|| {
            let home = env::var("HOME").ok()?;
            let profile = format!("{}/.nix-profile", home);
            run_command_with_timeout("nix-store", &["-qR", &profile], timeout)
        })?;
    let count = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    Some(count)
}

fn count_flatpak(timeout: Duration) -> Option<usize> {
    // Count all flatpak packages (apps + runtimes), like fastfetch
    let output = run_command_with_timeout("flatpak", &["list", "--columns=application"], timeout)?;
    let mut count = 0;
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.eq_ignore_ascii_case("application") {
            continue;
        }
        count += 1;
    }
    Some(count)
}

fn count_command_lines(command: &str, args: &[&str], timeout: Duration) -> Option<usize> {
    if !command_exists(command) {
        return None;
    }
    let output = run_command_with_timeout(command, args, timeout)?;
    Some(
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
    )
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') {
        return Path::new(command).exists();
    }
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|path| path.join(command).exists())
}
