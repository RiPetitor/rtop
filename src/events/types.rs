
use crossterm::event::KeyEvent;

use crate::data::gpu::GpuSnapshot;

/// Application events
#[derive(Debug)]
pub enum AppEvent {
    /// Terminal key press
    Key(KeyEvent),
    /// Periodic tick for refresh
    Tick,
    /// GPU data update from monitor thread
    GpuUpdate(GpuSnapshot),
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
