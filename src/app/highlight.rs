#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightMode {
    CurrentUser,
    NonRoot,
    Gui,
}

impl HighlightMode {
    pub fn label(self) -> &'static str {
        match self {
            HighlightMode::CurrentUser => "user",
            HighlightMode::NonRoot => "non-root",
            HighlightMode::Gui => "gui",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            HighlightMode::CurrentUser => HighlightMode::NonRoot,
            HighlightMode::NonRoot => HighlightMode::Gui,
            HighlightMode::Gui => HighlightMode::CurrentUser,
        }
    }
}

impl Default for HighlightMode {
    fn default() -> Self {
        HighlightMode::CurrentUser
    }
}
