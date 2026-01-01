use std::collections::{HashMap, HashSet};

use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::panel_block;
use super::theme::{COLOR_ACCENT, COLOR_MUTED};
use crate::app::App;
use crate::utils::fit_text;

#[derive(Clone, Debug)]
struct ProcNode {
    pid: u32,
    name: String,
    parent: Option<u32>,
    children: Vec<u32>,
}

enum ChildItem {
    Node(u32),
    Group { name: String, count: usize },
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block("Process Graph");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let Some(selected_pid) = app.selected_pid else {
        let paragraph = Paragraph::new("No process selected").alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
        return;
    };

    let nodes = build_nodes(app);
    if !nodes.contains_key(&selected_pid) {
        let paragraph = Paragraph::new("Process not available").alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
        return;
    }

    let path = build_path(selected_pid, &nodes);
    let root = path.first().copied().unwrap_or(selected_pid);
    let path_set = path.into_iter().collect::<HashSet<_>>();

    let max_lines = inner.height as usize;
    let width = inner.width as usize;
    let mut lines = Vec::new();
    let complete = render_node(
        root,
        "",
        true,
        true,
        &nodes,
        &path_set,
        selected_pid,
        &mut lines,
        max_lines,
        width,
    );

    if !complete {
        if !lines.is_empty() {
            lines.pop();
        }
        lines.push(Line::from(Span::styled(
            fit_text("... truncated ...", width),
            Style::default().fg(COLOR_MUTED),
        )));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No process data",
            Style::default().fg(COLOR_MUTED),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn build_nodes(app: &App) -> HashMap<u32, ProcNode> {
    let mut nodes = HashMap::new();
    for (pid, process) in app.system.processes() {
        let pid = pid.as_u32();
        let parent = process.parent().map(|parent| parent.as_u32());
        nodes.insert(
            pid,
            ProcNode {
                pid,
                name: process.name().to_string(),
                parent,
                children: Vec::new(),
            },
        );
    }

    let keys = nodes.keys().copied().collect::<Vec<_>>();
    for pid in keys {
        if let Some(parent) = nodes.get(&pid).and_then(|node| node.parent) {
            if let Some(parent_node) = nodes.get_mut(&parent) {
                parent_node.children.push(pid);
            }
        }
    }

    let name_map = nodes
        .iter()
        .map(|(pid, node)| (*pid, node.name.clone()))
        .collect::<HashMap<_, _>>();
    for node in nodes.values_mut() {
        node.children.sort_by(|a, b| {
            let name_a = name_map.get(a).map(String::as_str).unwrap_or("");
            let name_b = name_map.get(b).map(String::as_str).unwrap_or("");
            name_a.cmp(name_b).then_with(|| a.cmp(b))
        });
    }

    nodes
}

fn build_path(selected_pid: u32, nodes: &HashMap<u32, ProcNode>) -> Vec<u32> {
    let mut path = Vec::new();
    let mut current = Some(selected_pid);
    let mut visited = HashSet::new();
    while let Some(pid) = current {
        if !visited.insert(pid) {
            break;
        }
        path.push(pid);
        current = nodes
            .get(&pid)
            .and_then(|node| node.parent)
            .filter(|parent| nodes.contains_key(parent));
    }
    path.reverse();
    path
}

fn render_node(
    pid: u32,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    nodes: &HashMap<u32, ProcNode>,
    path_set: &HashSet<u32>,
    selected_pid: u32,
    lines: &mut Vec<Line<'static>>,
    max_lines: usize,
    width: usize,
) -> bool {
    if lines.len() >= max_lines {
        return false;
    }

    let Some(node) = nodes.get(&pid) else {
        return true;
    };

    let name = if node.name.is_empty() {
        "<unknown>"
    } else {
        node.name.as_str()
    };
    let label = format!("{name} ({})", node.pid);
    let text = render_line(prefix, is_last, is_root, &label, width);
    let style = if pid == selected_pid {
        Style::default()
            .fg(COLOR_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(Span::styled(text, style)));

    if lines.len() >= max_lines {
        return false;
    }

    let child_pids = if pid == selected_pid {
        node.children.clone()
    } else if path_set.contains(&pid) {
        node.children
            .iter()
            .copied()
            .filter(|child| path_set.contains(child))
            .collect()
    } else {
        node.children.clone()
    };

    let allow_group = pid == selected_pid || !path_set.contains(&pid);
    let items = build_child_items(&child_pids, nodes, allow_group);

    let next_prefix = if is_root {
        String::new()
    } else {
        let mut next = String::from(prefix);
        if is_last {
            next.push_str("    ");
        } else {
            next.push_str("|   ");
        }
        next
    };

    for (idx, item) in items.iter().enumerate() {
        let child_last = idx + 1 == items.len();
        match item {
            ChildItem::Node(child_pid) => {
                if !render_node(
                    *child_pid,
                    &next_prefix,
                    child_last,
                    false,
                    nodes,
                    path_set,
                    selected_pid,
                    lines,
                    max_lines,
                    width,
                ) {
                    return false;
                }
            }
            ChildItem::Group { name, count } => {
                if lines.len() >= max_lines {
                    return false;
                }
                let label = format!("{name} (x{count})");
                let text = render_line(&next_prefix, child_last, false, &label, width);
                lines.push(Line::from(Span::styled(
                    text,
                    Style::default().fg(COLOR_MUTED),
                )));
            }
        }
    }

    true
}

fn build_child_items(
    children: &[u32],
    nodes: &HashMap<u32, ProcNode>,
    allow_group: bool,
) -> Vec<ChildItem> {
    if !allow_group {
        return children.iter().copied().map(ChildItem::Node).collect();
    }

    let mut items = Vec::new();
    let mut idx = 0;
    while idx < children.len() {
        let pid = children[idx];
        let Some(node) = nodes.get(&pid) else {
            idx += 1;
            continue;
        };
        if !node.children.is_empty() {
            items.push(ChildItem::Node(pid));
            idx += 1;
            continue;
        }

        let name = node.name.clone();
        let mut count = 1;
        idx += 1;
        while idx < children.len() {
            let next_pid = children[idx];
            let Some(next_node) = nodes.get(&next_pid) else {
                idx += 1;
                continue;
            };
            if !next_node.children.is_empty() || next_node.name != name {
                break;
            }
            count += 1;
            idx += 1;
        }

        if count > 1 {
            items.push(ChildItem::Group { name, count });
        } else {
            items.push(ChildItem::Node(pid));
        }
    }

    items
}

fn render_line(prefix: &str, is_last: bool, is_root: bool, label: &str, width: usize) -> String {
    let connector = if is_root {
        ""
    } else if is_last {
        "\\-- "
    } else {
        "|-- "
    };
    fit_text(&format!("{prefix}{connector}{label}"), width)
}
