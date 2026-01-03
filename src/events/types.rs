use crossterm::event::{KeyEvent, MouseEvent};

/// Application events
#[derive(Debug)]
pub enum AppEvent {
    /// Terminal key press
    Key(KeyEvent),
    /// Mouse input
    Mouse(MouseEvent),
    /// Periodic tick for refresh
    Tick,
    /// Terminal resize
    Resize(u16, u16),
    /// Request to quit
    Quit,
}

/// Result of handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Continue running
    Continue,
    /// Exit the application
    Exit,
}

impl EventResult {
    pub fn should_exit(self) -> bool {
        matches!(self, EventResult::Exit)
    }
}
