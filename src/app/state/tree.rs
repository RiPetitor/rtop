use std::collections::{HashMap, HashSet};

use crate::data::ProcessRow;

pub(super) struct TreeLayout {
    pub(super) order: Vec<u32>,
    pub(super) labels: HashMap<u32, String>,
}

pub(super) fn build_tree_layout(
    parents: &HashMap<u32, Option<u32>>,
    rows: &HashMap<u32, ProcessRow>,
) -> TreeLayout {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, parent) in parents.iter() {
        if let Some(parent) = *parent {
            children.entry(parent).or_default().push(pid);
        }
    }
    for list in children.values_mut() {
        list.sort_unstable();
    }

    let mut roots = Vec::new();
    for (&pid, parent) in parents.iter() {
        let has_parent = parent
            .and_then(|parent| parents.contains_key(&parent).then_some(parent))
            .is_some();
        if !has_parent {
            roots.push(pid);
        }
    }
    roots.sort_unstable();

    let mut layout = TreeLayout {
        order: Vec::with_capacity(rows.len()),
        labels: HashMap::with_capacity(rows.len()),
    };
    let mut visited = HashSet::with_capacity(rows.len());

    for (idx, root) in roots.iter().enumerate() {
        let is_last = idx + 1 == roots.len();
        push_tree_layout(
            *root,
            "",
            is_last,
            true,
            &children,
            rows,
            &mut layout,
            &mut visited,
        );
    }

    layout
}

#[allow(clippy::too_many_arguments)]
fn push_tree_layout(
    pid: u32,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    children: &HashMap<u32, Vec<u32>>,
    rows: &HashMap<u32, ProcessRow>,
    layout: &mut TreeLayout,
    visited: &mut HashSet<u32>,
) {
    if !visited.insert(pid) {
        return;
    }
    let Some(row) = rows.get(&pid) else {
        return;
    };

    let connector = if is_root {
        ""
    } else if is_last {
        "\\- "
    } else {
        "|- "
    };
    let label = format!("{prefix}{connector}{}", row.name);
    layout.labels.insert(pid, label);
    layout.order.push(pid);

    let next_prefix = if is_root {
        String::new()
    } else if is_last {
        format!("{prefix}   ")
    } else {
        format!("{prefix}|  ")
    };

    if let Some(list) = children.get(&pid) {
        let last_index = list.len().saturating_sub(1);
        for (idx, child) in list.iter().enumerate() {
            push_tree_layout(
                *child,
                &next_prefix,
                idx == last_index,
                false,
                children,
                rows,
                layout,
                visited,
            );
        }
    }
}
