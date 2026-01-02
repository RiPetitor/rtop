use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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
        AppEvent::GpuUpdate(snapshot) => {
            app.apply_gpu_snapshot(snapshot);
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
        KeyCode::Char('q') => EventResult::Exit,
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
            } else {
                app.move_selection(-1);
            }
            EventResult::Continue
        }
        KeyCode::Down => {
            if app.view_mode == ViewMode::Container {
                app.move_container_selection(1);
            } else {
                app.move_selection(1);
            }
            EventResult::Continue
        }
        KeyCode::Esc | KeyCode::Char('b') => {
            if app.container_filter.is_some() {
                app.exit_container_drill();
            }
            EventResult::Continue
        }
        KeyCode::Left => {
            app.set_sort_key(app.sort_key.prev());
            EventResult::Continue
        }
        KeyCode::Right => {
            app.set_sort_key(app.sort_key.next());
            EventResult::Continue
        }
        KeyCode::Char(' ') => {
            app.toggle_sort_dir();
            EventResult::Continue
        }
        KeyCode::Enter => {
            if app.view_mode == ViewMode::Container {
                app.enter_container();
            } else {
                app.open_confirm();
            }
            EventResult::Continue
        }
        KeyCode::Char('c') => {
            app.set_sort_key(SortKey::Cpu);
            EventResult::Continue
        }
        KeyCode::Char('m') => {
            app.set_sort_key(SortKey::Mem);
            EventResult::Continue
        }
        KeyCode::Char('p') => {
            app.set_sort_key(SortKey::Pid);
            EventResult::Continue
        }
        KeyCode::Char('u') => {
            app.set_sort_key(SortKey::User);
            EventResult::Continue
        }
        KeyCode::Char('h') => {
            app.cycle_highlight_mode();
            EventResult::Continue
        }
        KeyCode::Char('n') => {
            app.set_sort_key(SortKey::Name);
            EventResult::Continue
        }
        KeyCode::Char('r') => {
            app.refresh();
            EventResult::Continue
        }
        KeyCode::Char('g') => {
            app.select_next_gpu();
            EventResult::Continue
        }
        KeyCode::Char('t') => {
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
            app.cycle_view_mode();
            EventResult::Continue
        }
        KeyCode::Char('G') => {
            app.select_prev_gpu();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_confirm_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
            app.cancel_confirm();
            EventResult::Continue
        }
        KeyCode::Enter | KeyCode::Char('y') => {
            app.confirm_kill();
            EventResult::Continue
        }
        _ => EventResult::Continue,
    }
}

fn handle_setup_key(app: &mut App, key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Esc | KeyCode::F(2) | KeyCode::Char('q') => {
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
        KeyCode::Esc | KeyCode::F(12) | KeyCode::Char('q') => {
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

    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
        && let Some(key) = app.sort_key_for_header_click(mouse.column, mouse.row)
    {
        if key == app.sort_key {
            app.toggle_sort_dir();
        } else {
            app.set_sort_key(key);
        }
    }

    EventResult::Continue
}
