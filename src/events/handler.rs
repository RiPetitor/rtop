use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::types::{AppEvent, EventResult};
use crate::app::App;
use crate::data::SortKey;

/// Handle an application event
pub fn handle_event(app: &mut App, event: AppEvent) -> EventResult {
    match event {
        AppEvent::Key(key) => handle_key(app, key),
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

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => EventResult::Exit,
        KeyCode::Char('q') => EventResult::Exit,
        KeyCode::Up => {
            app.move_selection(-1);
            EventResult::Continue
        }
        KeyCode::Down => {
            app.move_selection(1);
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
            app.open_confirm();
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
