use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use sysinfo::Pid;

use crate::app::App;
use crate::utils::run_command_with_timeout;

pub fn desktop_environment() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(desktop_environment_inner).clone()
}

fn desktop_environment_inner() -> Option<String> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("XDG_SESSION_DESKTOP"))
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .ok()?;
    let primary = desktop.split(':').next().unwrap_or(desktop.as_str());
    let lower = primary.to_ascii_lowercase();

    if lower.contains("kde") || lower.contains("plasma") {
        let version = command_version("plasmashell", &["--version"]);
        return Some(match version {
            Some(ver) => format!("KDE Plasma {ver}"),
            None => "KDE Plasma".to_string(),
        });
    }
    if lower.contains("gnome") {
        let version = command_version("gnome-shell", &["--version"]);
        return Some(match version {
            Some(ver) => format!("GNOME {ver}"),
            None => "GNOME".to_string(),
        });
    }
    if lower.contains("xfce") {
        return Some("XFCE".to_string());
    }
    if lower.contains("lxqt") {
        return Some("LXQt".to_string());
    }
    if lower.contains("lxde") {
        return Some("LXDE".to_string());
    }
    if lower.contains("cinnamon") {
        return Some("Cinnamon".to_string());
    }
    if lower.contains("mate") {
        return Some("MATE".to_string());
    }
    if lower.contains("budgie") {
        return Some("Budgie".to_string());
    }
    if lower.contains("deepin") {
        return Some("Deepin".to_string());
    }

    Some(primary.to_string())
}

pub fn window_manager(app: &App) -> Option<String> {
    let mut wm = None;
    for process in app.system.processes().values() {
        let name = process.name().to_string_lossy().to_ascii_lowercase();
        let detected = match name.as_str() {
            "kwin_wayland" | "kwin_x11" | "kwin" => Some("KWin"),
            "mutter" | "gnome-shell" => Some("Mutter"),
            "sway" => Some("Sway"),
            "hyprland" => Some("Hyprland"),
            "wayfire" => Some("Wayfire"),
            "river" => Some("River"),
            "labwc" => Some("LabWC"),
            "openbox" => Some("Openbox"),
            "i3" => Some("i3"),
            "bspwm" => Some("bspwm"),
            "awesome" => Some("Awesome"),
            "dwm" => Some("dwm"),
            _ => None,
        };
        if detected.is_some() {
            wm = detected.map(|value| value.to_string());
            break;
        }
    }
    let mut wm = wm?;
    if let Some(session) = session_type() {
        wm.push_str(" (");
        wm.push_str(session);
        wm.push(')');
    }
    Some(wm)
}

fn session_type() -> Option<&'static str> {
    let session = env::var("XDG_SESSION_TYPE").ok()?;
    if session.eq_ignore_ascii_case("wayland") {
        Some("Wayland")
    } else if session.eq_ignore_ascii_case("x11") {
        Some("X11")
    } else {
        None
    }
}

pub fn shell_name() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(shell_name_inner).clone()
}

fn shell_name_inner() -> Option<String> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(shell.as_str())
        .to_string();
    let version = match name.as_str() {
        "bash" => env::var("BASH_VERSION").ok(),
        "zsh" => env::var("ZSH_VERSION").ok(),
        "fish" => env::var("FISH_VERSION").ok(),
        "nu" | "nushell" => env::var("NU_VERSION").ok(),
        _ => None,
    }
    .and_then(|value| extract_version_token(&value));
    Some(match version {
        Some(version) => format!("{name} {version}"),
        None => name,
    })
}

pub fn terminal_name(app: &App) -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(|| terminal_name_inner(app)).clone()
}

fn terminal_name_inner(app: &App) -> Option<String> {
    if let Ok(term) = env::var("TERM_PROGRAM") {
        let version = env::var("TERM_PROGRAM_VERSION")
            .ok()
            .and_then(|value| extract_version_token(&value));
        let name = normalize_terminal_name(&term);
        return Some(match version {
            Some(version) => format!("{name} {version}"),
            None => name,
        });
    }
    if let Ok(term) = env::var("LC_TERMINAL") {
        let version = env::var("LC_TERMINAL_VERSION")
            .ok()
            .and_then(|value| extract_version_token(&value));
        let name = normalize_terminal_name(&term);
        return Some(match version {
            Some(version) => format!("{name} {version}"),
            None => name,
        });
    }

    let mut pid = Pid::from_u32(std::process::id());
    for _ in 0..8 {
        let process = app.system.process(pid)?;
        let name = process.name().to_string_lossy();
        let name = name.as_ref();
        if let Some(display) = known_terminal_name(name) {
            let version =
                terminal_version(name, process.exe().map(|path| path.to_path_buf()).as_ref());
            return Some(match version {
                Some(version) => format!("{display} {version}"),
                None => display,
            });
        }
        pid = process.parent()?;
    }
    None
}

fn known_terminal_name(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let display = match lower.as_str() {
        "ptyxis" => "Ptyxis",
        "gnome-terminal" | "gnome-terminal-server" => "GNOME Terminal",
        "kgx" => "GNOME Console",
        "konsole" => "Konsole",
        "alacritty" => "Alacritty",
        "kitty" => "Kitty",
        "wezterm" | "wezterm-gui" => "WezTerm",
        "foot" | "footclient" => "Foot",
        "tilix" => "Tilix",
        "terminator" => "Terminator",
        "xfce4-terminal" => "XFCE Terminal",
        "mate-terminal" => "MATE Terminal",
        "lxterminal" => "LXTerminal",
        "qterminal" => "QTerminal",
        "xterm" => "XTerm",
        "st" => "st",
        "urxvt" | "rxvt" => "rxvt",
        _ => return None,
    };
    Some(display.to_string())
}

fn normalize_terminal_name(name: &str) -> String {
    if let Some(display) = known_terminal_name(name) {
        return display;
    }
    let mut chars = name.chars();
    let mut output = String::new();
    if let Some(first) = chars.next() {
        output.push(first.to_ascii_uppercase());
        output.extend(chars);
    }
    output
}

fn terminal_version(name: &str, exe: Option<&PathBuf>) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let command = match lower.as_str() {
        "gnome-terminal-server" => "gnome-terminal",
        "wezterm-gui" => "wezterm",
        _ => name,
    };
    let command = if command == name {
        exe.and_then(|path| path.to_str()).unwrap_or(command)
    } else {
        command
    };
    command_version(command, &["--version"])
}

pub fn command_version(command: &str, args: &[&str]) -> Option<String> {
    let output = run_command_with_timeout(command, args, Duration::from_millis(400))?;
    extract_version_token(&output)
}

pub fn extract_version_token(value: &str) -> Option<String> {
    for token in value.split_whitespace() {
        let mut buf = String::new();
        let mut started = false;
        for ch in token.chars() {
            if ch.is_ascii_digit() {
                started = true;
                buf.push(ch);
            } else if started && ch == '.' {
                buf.push(ch);
            } else if started {
                break;
            }
        }
        if started {
            return Some(buf.trim_end_matches('.').to_string());
        }
    }
    None
}
