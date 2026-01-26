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
    pub repeat: Option<bool>,            // defaults to true
    pub cooldown_ms: Option<u32>,        // delay between repeats
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
    pub key: String, // XKB key name (e.g., "T", "Left", "XF86AudioRaiseVolume")
    pub properties: BindingProperties,
    pub action: BindingAction,
    #[allow(dead_code)]
    pub kdl_index: Option<usize>, // Position in the KDL binds block for editing
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

/// Which field is being edited in edit mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EditField {
    #[default]
    KeyCombo,
    ActionType,
    ActionValue,
    Repeat,
    AllowWhenLocked,
}

impl EditField {
    pub fn next(&self) -> Self {
        match self {
            EditField::KeyCombo => EditField::ActionType,
            EditField::ActionType => EditField::ActionValue,
            EditField::ActionValue => EditField::Repeat,
            EditField::Repeat => EditField::AllowWhenLocked,
            EditField::AllowWhenLocked => EditField::KeyCombo,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            EditField::KeyCombo => EditField::AllowWhenLocked,
            EditField::ActionType => EditField::KeyCombo,
            EditField::ActionValue => EditField::ActionType,
            EditField::Repeat => EditField::ActionValue,
            EditField::AllowWhenLocked => EditField::Repeat,
        }
    }
}

/// Type of action being edited
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ActionType {
    #[default]
    Spawn,      // Run a command
    SpawnSh,    // Run a shell command
    BuiltIn,    // Niri built-in action
}

impl ActionType {
    pub fn next(&self) -> Self {
        match self {
            ActionType::Spawn => ActionType::SpawnSh,
            ActionType::SpawnSh => ActionType::BuiltIn,
            ActionType::BuiltIn => ActionType::Spawn,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ActionType::Spawn => ActionType::BuiltIn,
            ActionType::SpawnSh => ActionType::Spawn,
            ActionType::BuiltIn => ActionType::SpawnSh,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActionType::Spawn => "Run Command",
            ActionType::SpawnSh => "Shell Command",
            ActionType::BuiltIn => "Built-in Action",
        }
    }
}

/// State for editing a keybinding
#[derive(Debug, Clone)]
pub struct EditMode {
    pub original_index: usize, // Index in the bindings list
    pub is_new: bool,          // True if adding new binding
    pub focused_field: EditField,
    pub key_combo: String,        // e.g., "Mod+Shift+T"
    pub key_combo_cursor: usize,  // Cursor position in key_combo
    pub action_type: ActionType,
    pub action_value: String,     // Command or action name
    pub action_value_cursor: usize, // Cursor position in action_value
    pub repeat: Option<bool>,
    pub allow_when_locked: Option<bool>,
}

impl EditMode {
    /// Create edit mode from an existing keybinding
    pub fn from_binding(index: usize, binding: &Keybinding) -> Self {
        let (action_type, action_value) = Self::action_to_parts(&binding.action);
        let key_combo = binding.combo();
        let key_combo_cursor = key_combo.len();
        let action_value_cursor = action_value.len();
        Self {
            original_index: index,
            is_new: false,
            focused_field: EditField::KeyCombo,
            key_combo,
            key_combo_cursor,
            action_type,
            action_value,
            action_value_cursor,
            repeat: binding.properties.repeat,
            allow_when_locked: binding.properties.allow_when_locked,
        }
    }

    /// Create edit mode for a new keybinding
    pub fn new_binding() -> Self {
        Self {
            original_index: 0,
            is_new: true,
            focused_field: EditField::KeyCombo,
            key_combo: String::new(),
            key_combo_cursor: 0,
            action_type: ActionType::Spawn,
            action_value: String::new(),
            action_value_cursor: 0,
            repeat: None,
            allow_when_locked: None,
        }
    }

    /// Insert a character at the current cursor position for the focused text field
    pub fn insert_char(&mut self, c: char) {
        match self.focused_field {
            EditField::KeyCombo => {
                self.key_combo.insert(self.key_combo_cursor, c);
                self.key_combo_cursor += 1;
            }
            EditField::ActionValue => {
                self.action_value.insert(self.action_value_cursor, c);
                self.action_value_cursor += 1;
            }
            _ => {}
        }
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        match self.focused_field {
            EditField::KeyCombo => {
                if self.key_combo_cursor > 0 {
                    self.key_combo_cursor -= 1;
                    self.key_combo.remove(self.key_combo_cursor);
                }
            }
            EditField::ActionValue => {
                if self.action_value_cursor > 0 {
                    self.action_value_cursor -= 1;
                    self.action_value.remove(self.action_value_cursor);
                }
            }
            _ => {}
        }
    }

    /// Move cursor left in the focused text field
    pub fn cursor_left(&mut self) {
        match self.focused_field {
            EditField::KeyCombo => {
                self.key_combo_cursor = self.key_combo_cursor.saturating_sub(1);
            }
            EditField::ActionValue => {
                self.action_value_cursor = self.action_value_cursor.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move cursor right in the focused text field
    pub fn cursor_right(&mut self) {
        match self.focused_field {
            EditField::KeyCombo => {
                self.key_combo_cursor = (self.key_combo_cursor + 1).min(self.key_combo.len());
            }
            EditField::ActionValue => {
                self.action_value_cursor = (self.action_value_cursor + 1).min(self.action_value.len());
            }
            _ => {}
        }
    }

    /// Move cursor to start of the focused text field
    pub fn cursor_home(&mut self) {
        match self.focused_field {
            EditField::KeyCombo => self.key_combo_cursor = 0,
            EditField::ActionValue => self.action_value_cursor = 0,
            _ => {}
        }
    }

    /// Move cursor to end of the focused text field
    pub fn cursor_end(&mut self) {
        match self.focused_field {
            EditField::KeyCombo => self.key_combo_cursor = self.key_combo.len(),
            EditField::ActionValue => self.action_value_cursor = self.action_value.len(),
            _ => {}
        }
    }

    /// Convert action to editable parts (type + value)
    fn action_to_parts(action: &BindingAction) -> (ActionType, String) {
        match action {
            BindingAction::Spawn(args) => {
                (ActionType::Spawn, args.join(" "))
            }
            BindingAction::SpawnSh(cmd) => {
                (ActionType::SpawnSh, cmd.clone())
            }
            BindingAction::Simple(name) => {
                (ActionType::BuiltIn, name.clone())
            }
            BindingAction::WithArg(name, arg) => {
                (ActionType::BuiltIn, format!("{name} {arg}"))
            }
        }
    }

    /// Convert edit state to a Keybinding
    pub fn to_keybinding(&self) -> Option<Keybinding> {
        if self.key_combo.is_empty() || self.action_value.is_empty() {
            return None;
        }

        let action = self.build_action()?;
        let (modifiers, key) = Modifiers::parse(&self.key_combo);

        Some(Keybinding {
            modifiers,
            key,
            properties: BindingProperties {
                repeat: self.repeat,
                cooldown_ms: None,
                allow_when_locked: self.allow_when_locked,
            },
            action,
            kdl_index: None,
        })
    }

    /// Build action from current edit state
    fn build_action(&self) -> Option<BindingAction> {
        let value = self.action_value.trim();
        if value.is_empty() {
            return None;
        }

        match self.action_type {
            ActionType::Spawn => {
                // Split by spaces, but respect quotes
                let args = parse_command_args(value);
                if args.is_empty() {
                    None
                } else {
                    Some(BindingAction::Spawn(args))
                }
            }
            ActionType::SpawnSh => {
                Some(BindingAction::SpawnSh(value.to_string()))
            }
            ActionType::BuiltIn => {
                // Parse as "action" or "action arg"
                let parts: Vec<&str> = value.splitn(2, ' ').collect();
                let action_name = parts[0];

                if parts.len() == 1 {
                    Some(BindingAction::Simple(action_name.to_string()))
                } else {
                    let arg_str = parts[1].trim();
                    let arg = if let Ok(n) = arg_str.parse::<i64>() {
                        BindingArg::Number(n)
                    } else if arg_str == "true" {
                        BindingArg::Bool(true)
                    } else if arg_str == "false" {
                        BindingArg::Bool(false)
                    } else {
                        BindingArg::String(arg_str.to_string())
                    };
                    Some(BindingAction::WithArg(action_name.to_string(), arg))
                }
            }
        }
    }

    /// Toggle repeat property
    pub fn toggle_repeat(&mut self) {
        self.repeat = match self.repeat {
            None => Some(false),       // Default (true) -> explicit false
            Some(false) => Some(true), // Explicit false -> explicit true
            Some(true) => None,        // Explicit true -> default
        };
    }

    /// Toggle allow-when-locked property
    pub fn toggle_allow_when_locked(&mut self) {
        self.allow_when_locked = match self.allow_when_locked {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };
    }

    /// Cycle action type forward
    pub fn next_action_type(&mut self) {
        self.action_type = self.action_type.next();
    }

    /// Cycle action type backward
    pub fn prev_action_type(&mut self) {
        self.action_type = self.action_type.prev();
    }
}

/// Parse command arguments, handling quoted strings
fn parse_command_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for c in s.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Status of a binding in the effective list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BindingStatus {
    Unchanged,
    Modified,
    Added,
}

/// A binding with its effective state for display
#[derive(Debug, Clone)]
pub struct EffectiveBinding {
    pub binding: Keybinding,
    pub original_index: Option<usize>, // None for added bindings
    pub status: BindingStatus,
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
    pub edit_mode: Option<EditMode>,
}

impl KeybindingsViewModel {
    /// Get effective bindings with pending changes applied
    pub fn effective_bindings(&self) -> Vec<EffectiveBinding> {
        let mut result = Vec::new();

        // Build a set of deleted indices
        let deleted: std::collections::HashSet<usize> = self.pending_changes
            .iter()
            .filter_map(|c| match c {
                KeybindingChange::Delete(idx) => Some(*idx),
                _ => None,
            })
            .collect();

        // Build a map of modified bindings
        let modified: std::collections::HashMap<usize, &Keybinding> = self.pending_changes
            .iter()
            .filter_map(|c| match c {
                KeybindingChange::Modify { index, new } => Some((*index, new)),
                _ => None,
            })
            .collect();

        // Process original bindings
        for (idx, binding) in self.bindings.iter().enumerate() {
            if deleted.contains(&idx) {
                continue; // Skip deleted
            }

            if let Some(new_binding) = modified.get(&idx) {
                result.push(EffectiveBinding {
                    binding: (*new_binding).clone(),
                    original_index: Some(idx),
                    status: BindingStatus::Modified,
                });
            } else {
                result.push(EffectiveBinding {
                    binding: binding.clone(),
                    original_index: Some(idx),
                    status: BindingStatus::Unchanged,
                });
            }
        }

        // Add new bindings
        for change in &self.pending_changes {
            if let KeybindingChange::Add(binding) = change {
                result.push(EffectiveBinding {
                    binding: binding.clone(),
                    original_index: None,
                    status: BindingStatus::Added,
                });
            }
        }

        result
    }

    /// Get filtered effective bindings based on search query
    pub fn filtered_bindings(&self) -> Vec<EffectiveBinding> {
        let effective = self.effective_bindings();
        if self.search_query.is_empty() {
            effective
        } else {
            effective
                .into_iter()
                .filter(|eb| eb.binding.matches_search(&self.search_query))
                .collect()
        }
    }

    /// Get the currently selected binding
    #[allow(dead_code)]
    pub fn selected_binding(&self) -> Option<Keybinding> {
        let filtered = self.filtered_bindings();
        filtered.get(self.selected_index).map(|eb| eb.binding.clone())
    }

    /// Get the currently selected effective binding (with status)
    pub fn selected_effective_binding(&self) -> Option<EffectiveBinding> {
        let filtered = self.filtered_bindings();
        filtered.get(self.selected_index).cloned()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_args() {
        assert_eq!(parse_command_args("alacritty"), vec!["alacritty"]);
        assert_eq!(parse_command_args("wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.1+"),
            vec!["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", "0.1+"]);
        assert_eq!(parse_command_args("sh -c 'echo hello'"),
            vec!["sh", "-c", "echo hello"]);
    }
}
