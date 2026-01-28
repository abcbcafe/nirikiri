use crossterm::event::KeyCode;

/// Available settings categories in the UI
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    #[default]
    Outputs,     // F1
    Keybindings, // F2
    Appearance,  // F3
}

impl Category {
    /// Get the category corresponding to a function key
    pub fn from_function_key(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::F(1) => Some(Category::Outputs),
            KeyCode::F(2) => Some(Category::Keybindings),
            KeyCode::F(3) => Some(Category::Appearance),
            _ => None,
        }
    }

    /// Get all categories in display order
    pub fn all() -> &'static [Category] {
        &[Category::Outputs, Category::Keybindings, Category::Appearance]
    }

    /// Get the display name for this category
    pub fn name(&self) -> &'static str {
        match self {
            Category::Outputs => "Outputs",
            Category::Keybindings => "Keybindings",
            Category::Appearance => "Appearance",
        }
    }

    /// Get the function key number for this category (1-indexed)
    pub fn function_key(&self) -> u8 {
        match self {
            Category::Outputs => 1,
            Category::Keybindings => 2,
            Category::Appearance => 3,
        }
    }

    /// Get the keybindings help text for this category's status bar
    pub fn keybinds(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Category::Outputs => &[
                ("q", "Quit"),
                ("Tab", "Select"),
                ("hjkl", "Move"),
                ("HJKL", "Snap"),
                ("n", "Normalize"),
                ("s", "Save"),
            ],
            Category::Keybindings => &[
                ("q", "Quit"),
                ("j/k", "Navigate"),
                ("/", "Search"),
                ("Enter", "Edit"),
                ("a", "Add"),
                ("d", "Delete"),
                ("s", "Save"),
            ],
            Category::Appearance => &[
                ("q", "Quit"),
                ("j/k", "Navigate"),
                ("Tab", "Expand/Collapse"),
                ("Enter", "Edit"),
                ("Space", "Toggle"),
                ("+/-", "Adjust"),
                ("s", "Save"),
            ],
        }
    }
}
