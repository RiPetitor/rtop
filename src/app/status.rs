use std::time::{Duration, Instant};

use ratatui::style::Style;

use crate::ui::theme::{COLOR_GOOD, COLOR_WARN};

pub struct StatusMessage {
    pub level: StatusLevel,
    pub text: String,
    pub expires_at: Instant,
}

impl StatusMessage {
    pub fn new(level: StatusLevel, text: String) -> Self {
        Self {
            level,
            text,
            expires_at: Instant::now() + Duration::from_secs(3),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

#[derive(Clone, Copy)]
pub enum StatusLevel {
    Info,
    Warn,
}

impl StatusLevel {
    pub fn style(self) -> Style {
        match self {
            StatusLevel::Info => Style::default().fg(COLOR_GOOD),
            StatusLevel::Warn => Style::default().fg(COLOR_WARN),
        }
    }
}
