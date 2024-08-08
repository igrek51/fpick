#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WindowFocus {
    Tree,
    ActionMenu,
}

impl Default for WindowFocus {
    fn default() -> Self {
        WindowFocus::Tree
    }
}
