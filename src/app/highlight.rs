#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HighlightMode {
    #[default]
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
