use std::fmt;

/// Modifier keys for a keybinding
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Modifiers {
    pub mod_key: bool, // Super/Logo key
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Modifiers {
    pub fn parse(combo: &str) -> (Self, String) {
        let mut mods = Modifiers::default();
        let parts: Vec<&str> = combo.split('+').collect();
        let key = parts.last().unwrap_or(&"").to_string();

        for part in &parts[..parts.len().saturating_sub(1)] {
            match part.to_lowercase().as_str() {
                "mod" | "super" | "logo" => mods.mod_key = true,
                "ctrl" | "control" => mods.ctrl = true,
                "shift" => mods.shift = true,
                "alt" => mods.alt = true,
                _ => {}
            }
        }

        (mods, key)
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.mod_key {
            parts.push("Mod");
        }
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        write!(f, "{}", parts.join("+"))
    }
}

/// Properties that can be set on a keybinding
#[derive(Debug, Clone, Default)]
pub struct BindingProperties {
    pub repeat: Option<bool>,         // defaults to true
    pub cooldown_ms: Option<u32>,     // delay between repeats
    pub allow_when_locked: Option<bool>, // allow when screen locked
}

#[allow(dead_code)]
impl BindingProperties {
    pub fn has_custom_properties(&self) -> bool {
        self.repeat.is_some() || self.cooldown_ms.is_some() || self.allow_when_locked.is_some()
    }
}

/// Action to perform when a keybinding is triggered
#[derive(Debug, Clone)]
pub enum BindingAction {
    /// Spawn a command with arguments: spawn "cmd" "arg1" "arg2"
    Spawn(Vec<String>),
    /// Spawn a shell command: spawn-sh "command"
    SpawnSh(String),
    /// Simple action without arguments: close-window, quit, etc.
    Simple(String),
    /// Action with a single argument: focus-workspace 1, set-column-width "50%"
    WithArg(String, BindingArg),
}

/// Argument for an action
#[derive(Debug, Clone)]
pub enum BindingArg {
    Number(i64),
    String(String),
    Bool(bool),
}

impl fmt::Display for BindingArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingArg::Number(n) => write!(f, "{n}"),
            BindingArg::String(s) => write!(f, "{s}"),
            BindingArg::Bool(b) => write!(f, "{b}"),
        }
    }
}

impl fmt::Display for BindingAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingAction::Spawn(args) => {
                if args.len() == 1 {
                    write!(f, "spawn {:?}", args[0])
                } else {
                    write!(f, "spawn {:?}", args.join(" "))
                }
            }
            BindingAction::SpawnSh(cmd) => write!(f, "spawn-sh {cmd:?}"),
            BindingAction::Simple(action) => write!(f, "{action}"),
            BindingAction::WithArg(action, arg) => write!(f, "{action} {arg}"),
        }
    }
}

impl BindingAction {
    /// Get a short description for display in the list
    pub fn short_description(&self) -> String {
        match self {
            BindingAction::Spawn(args) => {
                if let Some(cmd) = args.first() {
                    // Get just the command name (not full path)
                    let cmd_name = cmd.rsplit('/').next().unwrap_or(cmd);
                    if args.len() > 1 {
                        format!("{cmd_name} ...")
                    } else {
                        cmd_name.to_string()
                    }
                } else {
                    "spawn".to_string()
                }
            }
            BindingAction::SpawnSh(cmd) => {
                if cmd.len() > 20 {
                    format!("{}...", &cmd[..20])
                } else {
                    cmd.clone()
                }
            }
            BindingAction::Simple(action) => action.clone(),
            BindingAction::WithArg(action, arg) => format!("{action} {arg}"),
        }
    }

    /// Get the action category for grouping
    pub fn category(&self) -> &'static str {
        match self {
            BindingAction::Spawn(_) | BindingAction::SpawnSh(_) => "Program Execution",
            BindingAction::Simple(action) | BindingAction::WithArg(action, _) => {
                match action.as_str() {
                    "close-window" | "quit" | "power-off-monitors" => "Window Management",
                    a if a.starts_with("focus-") => "Focus",
                    a if a.starts_with("move-") => "Movement",
                    a if a.starts_with("set-") => "Layout",
                    a if a.starts_with("switch-") => "Workspace",
                    a if a.starts_with("consume-") || a.starts_with("expel-") => "Column",
                    "screenshot" | "screenshot-screen" | "screenshot-window" => "Screenshot",
                    _ => "Other",
                }
            }
        }
    }
}

/// A single keybinding entry
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub modifiers: Modifiers,
    pub key: String,                   // XKB key name (e.g., "T", "Left", "XF86AudioRaiseVolume")
    pub properties: BindingProperties,
    pub action: BindingAction,
    #[allow(dead_code)]
    pub kdl_index: Option<usize>,      // Position in the KDL binds block for editing
}

impl Keybinding {
    /// Get the full key combo string (e.g., "Mod+Shift+T")
    pub fn combo(&self) -> String {
        let mods = self.modifiers.to_string();
        if mods.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", mods, self.key)
        }
    }

    /// Check if this keybinding matches a search query
    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        let combo = self.combo().to_lowercase();
        let action_str = self.action.short_description().to_lowercase();

        combo.contains(&query) || action_str.contains(&query)
    }
}

/// Pending change to a keybinding
#[derive(Debug, Clone)]
#[allow(dead_code)] // Add and Modify variants are for future expansion
pub enum KeybindingChange {
    Add(Keybinding),
    Modify { index: usize, new: Keybinding },
    Delete(usize),
}

/// View model for the keybindings category
#[derive(Debug, Default)]
pub struct KeybindingsViewModel {
    pub bindings: Vec<Keybinding>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub pending_changes: Vec<KeybindingChange>,
    pub search_mode: bool,
}

impl KeybindingsViewModel {
    /// Get filtered bindings based on search query
    pub fn filtered_bindings(&self) -> Vec<(usize, &Keybinding)> {
        if self.search_query.is_empty() {
            self.bindings.iter().enumerate().collect()
        } else {
            self.bindings
                .iter()
                .enumerate()
                .filter(|(_, b)| b.matches_search(&self.search_query))
                .collect()
        }
    }

    /// Get the currently selected binding
    pub fn selected_binding(&self) -> Option<&Keybinding> {
        let filtered = self.filtered_bindings();
        filtered.get(self.selected_index).map(|(_, b)| *b)
    }

    /// Get the count of visible bindings
    pub fn visible_count(&self) -> usize {
        self.filtered_bindings().len()
    }

    /// Select next binding
    pub fn select_next(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    /// Select previous binding
    pub fn select_prev(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Set search query and reset selection
    pub fn set_search(&mut self, query: String) {
        self.search_query = query;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.search_mode = false;
    }

    /// Check if there are pending changes
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    /// Update scroll offset for visible area
    pub fn update_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }

        // Ensure selected item is visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }
}
