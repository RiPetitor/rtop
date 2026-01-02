use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::fs;
use sysinfo::System;

use super::panel_block;
use super::text::tr;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::App;
use crate::utils::{fit_text, format_bytes, format_duration};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block(tr(app.language, "System", "Система"));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let logo = select_logo();
    let min_info_width = 24;
    let max_logo_width = inner
        .width
        .saturating_sub(min_info_width)
        .max(10)
        .min(inner.width);
    let logo_width = (logo_max_width(logo.lines).min(max_logo_width as usize) as u16)
        .max(8)
        .min(inner.width);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(logo_width), Constraint::Min(0)])
        .split(inner);

    render_logo(frame, chunks[0], logo);
    render_info(frame, chunks[1], app);
}

fn render_logo(frame: &mut Frame, area: Rect, logo: &LogoSpec) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut lines = logo
        .lines
        .iter()
        .map(|line| parse_logo_line(line, logo.palette))
        .collect::<Vec<_>>();
    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let label_style = Style::default()
        .fg(COLOR_MUTED)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);
    let width = area.width.max(1) as usize;

    let unknown = tr(app.language, "unknown", "неизвестно");
    let host = System::host_name().unwrap_or_else(|| unknown.to_string());
    let user = app.current_user_name().unwrap_or(unknown);
    let user_host = format!("{user}@{host}");

    let os_name = System::name().unwrap_or_else(|| unknown.to_string());
    let os_version = System::os_version().unwrap_or_default();
    let os_line = if os_version.is_empty() {
        os_name
    } else {
        format!("{os_name} {os_version}")
    };

    let kernel = System::kernel_version().unwrap_or_else(|| unknown.to_string());
    let uptime = format_duration(System::uptime());
    let load = System::load_average();
    let cpu_brand = app
        .system
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_count = app.system.cpus().len();
    let arch = std::env::consts::ARCH;
    let total_mem = app.system.total_memory();
    let used_mem = app.system.used_memory();
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    let processes = app.system.processes().len();

    let gpu_label = if let Some((_idx, gpu)) = app.selected_gpu() {
        if gpu.name.is_empty() {
            gpu.vendor.clone().unwrap_or_else(|| "GPU".to_string())
        } else {
            gpu.name.clone()
        }
    } else {
        tr(app.language, "n/a", "н/д").to_string()
    };

    let mut lines = Vec::new();
    let label_user = format!("{:<6}", tr(app.language, "User", "Польз."));
    let label_host = format!("{:<6}", tr(app.language, "Host", "Хост"));
    let label_os = format!("{:<6}", tr(app.language, "OS", "ОС"));
    let label_kernel = format!("{:<6}", tr(app.language, "Kernel", "Ядро"));
    let label_arch = format!("{:<6}", tr(app.language, "Arch", "Арх"));
    let label_uptime = format!("{:<6}", tr(app.language, "Uptime", "Аптайм"));
    let label_load = format!("{:<6}", tr(app.language, "Load", "Нагр."));
    let label_cpu = format!("{:<6}", tr(app.language, "CPU", "CPU"));
    let label_memory = format!("{:<6}", tr(app.language, "Memory", "Память"));
    let label_swap = format!("{:<6}", tr(app.language, "Swap", "Swap"));
    let label_gpu = format!("{:<6}", tr(app.language, "GPU", "GPU"));
    let label_procs = format!("{:<6}", tr(app.language, "Procs", "Проц."));

    push_line(
        &mut lines,
        &label_user,
        user_host,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_host,
        host,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_os,
        os_line,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_kernel,
        kernel,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_arch,
        arch.to_string(),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_uptime,
        uptime,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_load,
        format!("{:.2} {:.2} {:.2}", load.one, load.five, load.fifteen),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_cpu,
        format!("{cpu_brand} ({cpu_count} cores)"),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_memory,
        format!("{} / {}", format_bytes(used_mem), format_bytes(total_mem)),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_swap,
        format!("{} / {}", format_bytes(used_swap), format_bytes(total_swap)),
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_gpu,
        gpu_label,
        width,
        label_style,
        value_style,
    );
    push_line(
        &mut lines,
        &label_procs,
        processes.to_string(),
        width,
        label_style,
        value_style,
    );

    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn push_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: String,
    width: usize,
    label_style: Style,
    value_style: Style,
) {
    let max_value = width.saturating_sub(label.len()).max(1);
    let value = fit_text(&value, max_value);
    lines.push(Line::from(vec![
        Span::styled(label.to_string(), label_style),
        Span::styled(value, value_style),
    ]));
}

struct LogoSpec {
    lines: &'static [&'static str],
    palette: &'static [Color],
}

const LOGO_RTOP: &[&str] = &[
    "     ____",
    "    / __ \\",
    "   / /_/ /",
    "  / ____/",
    " /_/",
    "  rtop",
];
const LOGO_ARCH: &[&str] = &[
    "                  -`",
    "                 .o+`",
    "                `ooo/",
    "               `+oooo:",
    "              `+oooooo:",
    "              -+oooooo+:",
    "            `/:-:++oooo+:",
    "           `/++++/+++++++:",
    "          `/++++++++++++++:",
    "         `/+++o$2oooooooo$1oooo/`",
    "        ./$2ooosssso++osssssso$1+`",
    "$2       .oossssso-````/ossssss+`",
    "      -osssssso.      :ssssssso.",
    "     :osssssss/        osssso+++.",
    "    /ossssssss/        +ssssooo/-",
    "  `/ossssso+/:-        -:/+osssso+-",
    " `+sso+:-`                 `.-/+oso:",
    "`++:.                           `-/+/",
    ".`                                 `/",
];
const LOGO_DEBIAN: &[&str] = &[
    "        $2_,met$$$$$$$$$$gg.",
    "     ,g$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$P.",
    "   ,g$$$$P\"\"       \"\"\"Y$$$$.\".",
    "  ,$$$$P'              `$$$$$$.",
    "',$$$$P       ,ggs.     `$$$$b:",
    "`d$$$$'     ,$P\"'   $1.$2    $$$$$$",
    " $$$$P      d$'     $1,$2    $$$$P",
    " $$$$:      $$$.   $1-$2    ,d$$$$'",
    " $$$$;      Y$b._   _,d$P'",
    " Y$$$$.    $1`.$2`\"Y$$$$$$$$P\"'",
    " `$$$$b      $1\"-.__",
    "  $2`Y$$$$b",
    "   `Y$$$$.",
    "     `$$$$b.",
    "       `Y$$$$b.",
    "         `\"Y$$b._",
    "             `\"\"\"\"",
];
const LOGO_FEDORA: &[&str] = &[
    "             .',;::::;,'.",
    "         .';:cccccccccccc:;,.",
    "      .;cccccccccccccccccccccc;.",
    "    .:cccccccccccccccccccccccccc:.",
    "  .;ccccccccccccc;$2.:dddl:.$1;ccccccc;.",
    " .:ccccccccccccc;$2OWMKOOXMWd$1;ccccccc:.",
    ".:ccccccccccccc;$2KMMc$1;cc;$2xMMc$1;ccccccc:.",
    ",cccccccccccccc;$2MMM.$1;cc;$2;WW:$1;cccccccc,",
    ":cccccccccccccc;$2MMM.$1;cccccccccccccccc:",
    ":ccccccc;$2oxOOOo$1;$2MMM000k.$1;cccccccccccc:",
    "cccccc;$20MMKxdd:$1;$2MMMkddc.$1;cccccccccccc;",
    "ccccc;$2XMO'$1;cccc;$2MMM.$1;cccccccccccccccc'",
    "ccccc;$2MMo$1;ccccc;$2MMW.$1;ccccccccccccccc;",
    "ccccc;$20MNc.$1ccc$2.xMMd$1;ccccccccccccccc;",
    "cccccc;$2dNMWXXXWM0:$1;cccccccccccccc:,",
    "cccccccc;$2.:odl:.$1;cccccccccccccc:,.",
    "ccccccccccccccccccccccccccccc:'.",
    ":ccccccccccccccccccccccc:;,..",
    " ':cccccccccccccccc::;,. ",
];
const LOGO_WINDOWS: &[&str] = &[
    "$1        ,.=:!!t3Z3z.,",
    "       :tt:::tt333EE3",
    "$1       Et:::ztt33EEEL$2 @Ee.,      ..,",
    "$1      ;tt:::tt333EE7$2 ;EEEEEEttttt33#",
    "$1     :Et:::zt333EEQ.$2 $EEEEEttttt33QL",
    "$1     it::::tt333EEF$2 @EEEEEEttttt33F",
    "$1    ;3=*^```\"*4EEV$2 :EEEEEEttttt33@.",
    "$3    ,.=::::!t=., $1`$2 @EEEEEEtttz33QF",
    "$3   ;::::::::zt33)$2   \"4EEEtttji3P*",
    "$3  :t::::::::tt33.$4:Z3z..$2  ``$4 ,..g.",
    "$3  i::::::::zt33F$4 AEEEtttt::::ztF",
    "$3 ;:::::::::t33V$4 ;EEEttttt::::t3",
    "$3 E::::::::zt33L$4 @EEEtttt::::z3F",
    "$3{3=*^```\"*4E3)$4 ;EEEtttt:::::tZ`",
    "$3             `$4 :EEEEtttt::::z7",
    "                 \"VEzjt:;;z>*`",
];
const LOGO_MACOS: &[&str] = &[
    "                     $1..'",
    "                 ,xNMM.",
    "               .OMMMMo",
    "               lMM\"",
    "     .;loddo:.  .olloddol;.",
    "   cKMMMMMMMMMMNWMMMMMMMMMM0:",
    " $2.KMMMMMMMMMMMMMMMMMMMMMMMWd.",
    " XMMMMMMMMMMMMMMMMMMMMMMMX.",
    "$3;MMMMMMMMMMMMMMMMMMMMMMMM:",
    ":MMMMMMMMMMMMMMMMMMMMMMMM:",
    ".MMMMMMMMMMMMMMMMMMMMMMMX.",
    " kMMMMMMMMMMMMMMMMMMMMMMMMWd.",
    " $4'XMMMMMMMMMMMMMMMMMMMMMMMMMMk",
    "  'XMMMMMMMMMMMMMMMMMMMMMMMMK.",
    "    $5kMMMMMMMMMMMMMMMMMMMMMMd",
    "     ;KMMMMMMMWXXWMMMMMMMk.",
    "       \"cooc*\"    \"*coo'",
];

const PALETTE_RTOP: [Color; 1] = [COLOR_ACCENT];
const PALETTE_ARCH: [Color; 2] = [Color::Rgb(30, 144, 210), Color::Rgb(121, 196, 236)];
const PALETTE_DEBIAN: [Color; 2] = [Color::Rgb(215, 10, 83), Color::Rgb(170, 25, 70)];
const PALETTE_FEDORA: [Color; 2] = [Color::Rgb(60, 113, 187), Color::Rgb(110, 170, 255)];
const PALETTE_WINDOWS: [Color; 4] = [
    Color::Rgb(0, 120, 215),
    Color::Rgb(0, 180, 200),
    Color::Rgb(255, 185, 0),
    Color::Rgb(255, 140, 0),
];
const PALETTE_MACOS: [Color; 5] = [
    Color::Rgb(160, 160, 160),
    Color::Rgb(255, 94, 0),
    Color::Rgb(255, 177, 0),
    Color::Rgb(60, 199, 63),
    Color::Rgb(0, 122, 255),
];

static LOGO_SPEC_RTOP: LogoSpec = LogoSpec {
    lines: LOGO_RTOP,
    palette: &PALETTE_RTOP,
};
static LOGO_SPEC_ARCH: LogoSpec = LogoSpec {
    lines: LOGO_ARCH,
    palette: &PALETTE_ARCH,
};
static LOGO_SPEC_DEBIAN: LogoSpec = LogoSpec {
    lines: LOGO_DEBIAN,
    palette: &PALETTE_DEBIAN,
};
static LOGO_SPEC_FEDORA: LogoSpec = LogoSpec {
    lines: LOGO_FEDORA,
    palette: &PALETTE_FEDORA,
};
static LOGO_SPEC_WINDOWS: LogoSpec = LogoSpec {
    lines: LOGO_WINDOWS,
    palette: &PALETTE_WINDOWS,
};
static LOGO_SPEC_MACOS: LogoSpec = LogoSpec {
    lines: LOGO_MACOS,
    palette: &PALETTE_MACOS,
};

fn select_logo() -> &'static LogoSpec {
    let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
    let mut os_id = os_name.to_ascii_lowercase();
    if os_id.contains("linux")
        && let Some(linux_id) = linux_os_id()
    {
        os_id = linux_id;
    }

    if os_id.contains("arch") {
        return &LOGO_SPEC_ARCH;
    }
    if os_id.contains("fedora") {
        return &LOGO_SPEC_FEDORA;
    }
    if os_id.contains("debian") {
        return &LOGO_SPEC_DEBIAN;
    }
    if os_id.contains("windows") {
        return &LOGO_SPEC_WINDOWS;
    }
    if os_id.contains("macos")
        || os_id.contains("darwin")
        || os_id.contains("os x")
        || os_id.contains("mac")
    {
        return &LOGO_SPEC_MACOS;
    }

    &LOGO_SPEC_RTOP
}

fn linux_os_id() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            return Some(value.trim_matches('\"').to_ascii_lowercase());
        }
    }
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            return Some(value.trim_matches('\"').to_ascii_lowercase());
        }
    }
    None
}

fn logo_max_width(lines: &[&str]) -> usize {
    lines
        .iter()
        .map(|line| stripped_logo_width(line))
        .max()
        .unwrap_or(0)
}

fn stripped_logo_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$'
            && let Some(next) = chars.peek()
            && next.is_ascii_digit()
        {
            chars.next();
            continue;
        }
        width += 1;
    }
    width
}

fn parse_logo_line(line: &str, palette: &[Color]) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_style = Style::default().fg(COLOR_ACCENT);
    let mut buffer = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$'
            && let Some(next) = chars.peek()
            && next.is_ascii_digit()
        {
            if !buffer.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buffer), current_style));
            }
            let digit = chars.next().unwrap();
            if digit == '0' {
                current_style = Style::default().fg(COLOR_ACCENT);
            } else {
                let idx = (digit as u8 - b'1') as usize;
                let color = palette.get(idx).copied().unwrap_or(COLOR_ACCENT);
                current_style = Style::default().fg(color);
            }
            continue;
        }
        buffer.push(ch);
    }

    if !buffer.is_empty() {
        spans.push(Span::styled(buffer, current_style));
    }
    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    Line::from(spans)
}
