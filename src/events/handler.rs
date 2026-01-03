use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;

use super::types::{AppEvent, EventResult};
use crate::app::{App, ViewMode};
use crate::data::SortKey;

/// Handle an application event
pub fn handle_event(app: &mut App, event: AppEvent) -> EventResult {
    match event {
        AppEvent::Key(key) => handle_key(app, key),
        AppEvent::Mouse(mouse) => handle_mouse(app, mouse),
        AppEvent::Tick => {
            app.refresh();
            EventResult::Continue
        }
        AppEvent::Resize(_, _) => {
            // UI will handle resize automatically
            EventResult::Continue
        }
        AppEvent::Quit => EventResult::Exit,
    }
}

/// Handle a key event, returns EventResult
pub fn handle_key(app: &mut App, key: KeyEvent) -> EventResult {
    if app.confirm.is_some() {
        return handle_confirm_key(app, key);
    }
    if app.show_setup {
        return handle_setup_key(app, key);
    }
    if app.show_help {
        return handle_help_key(app, key);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('с') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('q') | KeyCode::Char('й') => EventResult::Exit,
        KeyCode::F(2) => {
            app.toggle_setup();
            EventResult::Continue
        }
        KeyCode::F(12) => {
            app.toggle_help();
            EventResult::Continue
        }
        KeyCode::Up => {
            if app.view_mode == ViewMode::Container {
                app.move_container_selection(-1);
            } else if app.view_mode == ViewMode::GpuFocus {
                app.move_gpu_process_selection(-1);
            } else {
                app.move_selection(-1);
            }
            EventResult::Continue
        }
        KeyCode::Down => {
            if app.view_mode == ViewMode::Container {
                app.move_container_selection(1);
            } else if app.view_mode == ViewMode::GpuFocus {
                app.move_gpu_process_selection(1);
            } else {
                app.move_selection(1);
            }
            EventResult::Continue
        }
        KeyCode::Home => {
            if app.view_mode == ViewMode::GpuFocus {
                app.select_gpu_process_first();
            } else if matches!(app.view_mode, ViewMode::Overview | ViewMode::Processes) {
                app.select_process_row(0);
            }
            EventResult::Continue
        }
        KeyCode::End => {
            if app.view_mode == ViewMode::GpuFocus {
                app.select_gpu_process_last();
            } else if matches!(app.view_mode, ViewMode::Overview | ViewMode::Processes) {
                let last = app.rows.len().saturating_sub(1);
                app.select_process_row(last);
            }
            EventResult::Continue
        }
        KeyCode::PageUp => {
            if app.view_mode == ViewMode::GpuFocus {
                let delta = page_delta(app.gpu_process_body);
                if delta > 0 {
                    app.move_gpu_process_selection(-delta);
                }
            } else if matches!(app.view_mode, ViewMode::Overview | ViewMode::Processes) {
                let delta = page_delta(app.process_body);
                if delta > 0 {
                    app.move_selection(-delta);
                }
            }
            EventResult::Continue
        }
        KeyCode::PageDown => {
            if app.view_mode == ViewMode::GpuFocus {
                let delta = page_delta(app.gpu_process_body);
                if delta > 0 {
                    app.move_gpu_process_selection(delta);
                }
            } else if matches!(app.view_mode, ViewMode::Overview | ViewMode::Processes) {
                let delta = page_delta(app.process_body);
                if delta > 0 {
                    app.move_selection(delta);
                }
            }
            EventResult::Continue
        }
        KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('и') => {
            if app.container_filter.is_some() {
                app.exit_container_drill();
            } else if app.view_mode == ViewMode::Overview && app.processes_expanded {
                app.collapse_processes();
            } else if app.view_mode == ViewMode::GpuFocus && app.gpu_panel_expanded {
                app.collapse_gpu_panel();
            } else if app.view_mode != ViewMode::Overview {
                app.set_view_mode(ViewMode::Overview);
            }
            EventResult::Continue
        }
        KeyCode::Left => {
            if app.view_mode == ViewMode::GpuFocus {
                app.set_gpu_process_sort_key(app.gpu_process_sort_key.prev());
            } else {
                app.set_sort_key(app.sort_key.prev());
            }
            EventResult::Continue
        }
        KeyCode::Right => {
            if app.view_mode == ViewMode::GpuFocus {
                app.set_gpu_process_sort_key(app.gpu_process_sort_key.next());
            } else {
                app.set_sort_key(app.sort_key.next());
            }
            EventResult::Continue
        }
        KeyCode::Char(' ') => {
            if app.view_mode == ViewMode::GpuFocus {
                app.toggle_gpu_process_sort_dir();
            } else {
                app.toggle_sort_dir();
            }
            EventResult::Continue
        }
        KeyCode::Enter => {
            if app.view_mode == ViewMode::Container {
                app.enter_container();
            } else if app.view_mode == ViewMode::Overview {
                if app.processes_expanded {
                    // В развёрнутом режиме - kill процесса
                    app.open_confirm();
                } else {
                    // В обычном режиме - развернуть Processes
                    app.expand_processes();
                }
            } else if app.view_mode == ViewMode::GpuFocus {
                if app.gpu_panel_expanded {
                    // В развёрнутом режиме - kill процесса
                    if let Some(pid) = app.selected_gpu_process_pid() {
                        app.open_confirm_for_pid(pid);
                    } else {
                        app.set_status(
                            crate::app::StatusLevel::Warn,
                            "Select a GPU process first".to_string(),
                        );
                    }
                } else {
                    // В обычном режиме - развернуть панель
                    app.expand_gpu_panel();
                }
            } else {
                app.open_confirm();
            }
            EventResult::Continue
        }
        KeyCode::Char('c') | KeyCode::Char('с') => {
            app.set_sort_key(SortKey::Cpu);
            EventResult::Continue
        }
        KeyCode::Char('m') | KeyCode::Char('ь') => {
            app.set_sort_key(SortKey::Mem);
            EventResult::Continue
        }
        KeyCode::Char('p') | KeyCode::Char('з') => {
            app.set_sort_key(SortKey::Pid);
            EventResult::Continue
        }
        KeyCode::Char('u') | KeyCode::Char('г') => {
            app.set_sort_key(SortKey::User);
            EventResult::Continue
        }
        KeyCode::Char('h') | KeyCode::Char('р') => {
            app.cycle_highlight_mode();
            EventResult::Continue
        }
        KeyCode::Char('n') | KeyCode::Char('т') => {
            app.set_sort_key(SortKey::Name);
            EventResult::Continue
        }
        KeyCode::Char('r') | KeyCode::Char('к') => {
            app.refresh();
            EventResult::Continue
        }
        KeyCode::Char('g') | KeyCode::Char('п') => {
            app.select_next_gpu();
            EventResult::Continue
        }
        KeyCode::Char('t') | KeyCode::Char('е') => {
            app.toggle_tree_view();
            EventResult::Continue
        }
        KeyCode::Char('1') => {
            app.set_view_mode(ViewMode::Overview);
            EventResult::Continue
        }
        KeyCode::Char('2') => {
            app.set_view_mode(ViewMode::SystemInfo);
            EventResult::Continue
        }
        KeyCode::Char('3') => {
            app.set_view_mode(ViewMode::GpuFocus);
            EventResult::Continue
        }
        KeyCode::Char('4') => {
            app.set_view_mode(ViewMode::Container);
            EventResult::Continue
        }
        KeyCode::Char('5') => {
            app.set_view_mode(ViewMode::Processes);
            EventResult::Continue
        }
        KeyCode::Tab => {
            // Tab переключает панели внутри текущей вкладки
            if app.view_mode == ViewMode::Overview && !app.processes_expanded {
                app.toggle_processes_focus();
            } else if app.view_mode == ViewMode::GpuFocus && !app.gpu_panel_expanded {
                app.toggle_gpu_focus_panel();
            }
            // Переключение вкладок - только цифрами (1-5)
            EventResult::Continue
        }
        KeyCode::BackTab => {
            if app.view_mode == ViewMode::Overview && !app.processes_expanded {
                app.toggle_processes_focus();
            } else if app.view_mode == ViewMode::GpuFocus && !app.gpu_panel_expanded {
                app.toggle_gpu_focus_panel();
            }
            EventResult::Continue
        }
        KeyCode::Char('G') | KeyCode::Char('П') => {
            app.select_prev_gpu();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_confirm_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('с') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Esc
        | KeyCode::Char('n')
        | KeyCode::Char('т')
        | KeyCode::Char('q')
        | KeyCode::Char('й') => {
            app.cancel_confirm();
            EventResult::Continue
        }
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('н') => {
            app.confirm_kill();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_setup_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('с') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Esc | KeyCode::F(2) | KeyCode::Char('q') | KeyCode::Char('й') => {
            app.toggle_setup();
            EventResult::Continue
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
            app.toggle_language();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_help_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('с') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Esc | KeyCode::F(12) | KeyCode::Char('q') | KeyCode::Char('й') => {
            app.toggle_help();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> EventResult {
    if app.tree_view || app.show_help || app.show_setup || app.confirm.is_some() {
        return EventResult::Continue;
    }

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(key) = app.sort_key_for_header_click(mouse.column, mouse.row) {
                if key == app.sort_key {
                    app.toggle_sort_dir();
                } else {
                    app.set_sort_key(key);
                }
                return EventResult::Continue;
            }

            if let Some(key) = app.gpu_sort_key_for_header_click(mouse.column, mouse.row) {
                if key == app.gpu_process_sort_key {
                    app.toggle_gpu_process_sort_dir();
                } else {
                    app.set_gpu_process_sort_key(key);
                }
                return EventResult::Continue;
            }

            if let Some(body) = app.process_body
                && rect_contains(body, mouse.column, mouse.row)
            {
                let row_index = (mouse.row - body.y) as usize;
                let index = app.scroll.saturating_add(row_index);
                if index < app.rows.len() {
                    app.select_process_row(index);
                }
                return EventResult::Continue;
            }

            if let Some(body) = app.gpu_process_body
                && rect_contains(body, mouse.column, mouse.row)
            {
                let row_index = (mouse.row - body.y) as usize;
                let index = app.gpu_process_scroll.saturating_add(row_index);
                if let Some(pid) = app.gpu_process_order.get(index).copied() {
                    app.select_process_pid(pid);
                }
            }
        }
        MouseEventKind::ScrollUp => {
            handle_scroll(app, mouse.column, mouse.row, -1);
        }
        MouseEventKind::ScrollDown => {
            handle_scroll(app, mouse.column, mouse.row, 1);
        }
        _ => {}
    }

    EventResult::Continue
}

fn handle_scroll(app: &mut App, column: u16, row: u16, delta: i32) {
    if app.view_mode == ViewMode::GpuFocus {
        if let Some(body) = app.gpu_process_body
            && rect_contains(body, column, row)
        {
            app.move_gpu_process_selection(delta);
        }
        return;
    }

    if matches!(app.view_mode, ViewMode::Overview | ViewMode::Processes) {
        if let Some(body) = app.process_body
            && rect_contains(body, column, row)
        {
            app.move_selection(delta);
        }
    }
}

fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    row >= rect.y
        && row < rect.y.saturating_add(rect.height)
        && column >= rect.x
        && column < rect.x.saturating_add(rect.width)
}

fn page_delta(body: Option<Rect>) -> i32 {
    let Some(body) = body else {
        return 0;
    };
    let rows = body.height as usize;
    if rows == 0 {
        return 0;
    }
    rows.saturating_sub(1).max(1) as i32
}
