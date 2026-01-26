use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::time::Duration;

use crate::category::Category;
use crate::config::{
    get_configured_positions, load_config, parse_keybindings, write_keybindings, write_positions,
};
use crate::ipc::NiriClient;
use crate::message::Message;
use crate::model::{
    ConfigDocument, EditField, EditMode, KeybindingChange, KeybindingsViewModel, OutputViewModel,
};
use crate::update::update_output;
use crate::view::{
    KeybindingDetailWidget, KeybindingEditWidget, KeybindingsListWidget, OutputInfoWidget,
    OutputListWidget, StatusBarWidget, TabBarWidget,
};
use crate::widgets::{CanvasViewport, MonitorCanvasWidget};

/// Main application state
pub struct App {
    pub current_category: Category,
    pub view_model: OutputViewModel,
    pub keybindings_view_model: KeybindingsViewModel,
    pub config: Option<ConfigDocument>,
    pub viewport: CanvasViewport,
    pub error: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            current_category: Category::default(),
            view_model: OutputViewModel::default(),
            keybindings_view_model: KeybindingsViewModel::default(),
            config: None,
            viewport: CanvasViewport::default(),
            error: None,
            should_quit: false,
        };

        // Initialize
        app.load_outputs()?;
        app.load_config();

        Ok(app)
    }

    fn load_outputs(&mut self) -> Result<()> {
        let mut client = NiriClient::connect()?;
        self.view_model.outputs = client.get_outputs()?;
        Ok(())
    }

    fn load_config(&mut self) {
        match load_config() {
            Ok(config) => {
                // Mark outputs that have config entries
                let positions = get_configured_positions(&config);
                for (name, _) in &positions {
                    if let Some(output) = self.view_model.outputs.iter_mut().find(|o| &o.name == name)
                    {
                        output.configured = true;
                    }
                }

                // Load keybindings
                self.keybindings_view_model.bindings = parse_keybindings(&config);

                self.config = Some(config);
            }
            Err(e) => {
                self.error = Some(format!("Failed to load config: {e}"));
            }
        }
    }

    /// Process a message and update state
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Quit => {
                self.should_quit = true;
            }
            Message::SwitchCategory(category) => {
                self.current_category = category;
                self.error = None;
            }
            Message::PanCanvas { .. } => {
                // Panning removed - view auto-fits all monitors
            }
            Message::ZoomIn => {
                self.viewport.zoom_in();
            }
            Message::ZoomOut => {
                self.viewport.zoom_out();
            }
            Message::ResetView => {
                self.viewport.reset();
            }
            Message::Save => {
                self.save_config();
            }
            Message::Reload => {
                self.view_model.clear_pending_changes();
                self.keybindings_view_model.pending_changes.clear();
                if let Err(e) = self.load_outputs() {
                    self.error = Some(format!("Failed to reload: {e}"));
                } else {
                    self.load_config();
                }
            }
            Message::PreviewChanges => {
                self.preview_changes();
            }
            Message::RevertPreview => {
                self.view_model.clear_pending_changes();
            }
            Message::Error(e) => {
                self.error = Some(e);
            }
            Message::ClearError => {
                self.error = None;
            }
            Message::RefreshOutputs => {
                if let Err(e) = self.load_outputs() {
                    self.error = Some(format!("Failed to refresh: {e}"));
                }
            }
            // Keybindings navigation
            Message::SelectNextKeybinding => {
                self.keybindings_view_model.select_next();
            }
            Message::SelectPrevKeybinding => {
                self.keybindings_view_model.select_prev();
            }
            Message::SelectKeybinding(idx) => {
                let count = self.keybindings_view_model.visible_count();
                if idx < count {
                    self.keybindings_view_model.selected_index = idx;
                }
            }
            // Keybindings search
            Message::StartSearch => {
                self.keybindings_view_model.search_mode = true;
            }
            Message::UpdateSearch(query) => {
                self.keybindings_view_model.set_search(query);
            }
            Message::ClearSearch => {
                self.keybindings_view_model.clear_search();
            }
            // Keybindings editing
            Message::StartEdit => {
                self.start_edit_keybinding();
            }
            Message::CancelEdit => {
                self.keybindings_view_model.edit_mode = None;
                self.error = None;
            }
            Message::ConfirmEdit => {
                self.confirm_edit_keybinding();
            }
            Message::AddKeybinding => {
                self.keybindings_view_model.edit_mode = Some(EditMode::new_binding());
                self.error = None;
            }
            Message::DeleteKeybinding => {
                self.delete_selected_keybinding();
            }
            // Output-related messages
            msg => {
                update_output(&mut self.view_model, &msg);
            }
        }
    }

    fn save_config(&mut self) {
        match self.current_category {
            Category::Outputs => self.save_output_config(),
            Category::Keybindings => self.save_keybindings_config(),
        }
    }

    fn save_output_config(&mut self) {
        if !self.view_model.has_pending_changes() {
            return;
        }

        if let Some(config) = &mut self.config {
            match write_positions(config, &self.view_model.pending_changes) {
                Ok(()) => {
                    // Apply pending changes to outputs
                    for (name, pos) in &self.view_model.pending_changes {
                        if let Some(output) =
                            self.view_model.outputs.iter_mut().find(|o| &o.name == name)
                        {
                            output.position = *pos;
                            output.configured = true;
                        }
                    }
                    self.view_model.clear_pending_changes();
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to save: {e}"));
                }
            }
        } else {
            self.error = Some("No config loaded".to_string());
        }
    }

    fn save_keybindings_config(&mut self) {
        if !self.keybindings_view_model.has_pending_changes() {
            return;
        }

        if let Some(config) = &mut self.config {
            match write_keybindings(config, &self.keybindings_view_model.pending_changes) {
                Ok(()) => {
                    // Reload keybindings from saved config
                    self.keybindings_view_model.bindings = parse_keybindings(config);
                    self.keybindings_view_model.pending_changes.clear();
                    self.keybindings_view_model.selected_index = 0;
                    self.error = None;

                    // Tell niri to reload its config so keybindings take effect
                    if let Err(e) = NiriClient::connect().and_then(|mut c| c.reload_config()) {
                        self.error = Some(format!("Saved, but failed to reload niri config: {e}"));
                    }
                }
                Err(e) => {
                    self.error = Some(format!("Failed to save keybindings: {e}"));
                }
            }
        } else {
            self.error = Some("No config loaded".to_string());
        }
    }

    fn delete_selected_keybinding(&mut self) {
        let filtered = self.keybindings_view_model.filtered_bindings();
        if let Some(eb) = filtered.get(self.keybindings_view_model.selected_index) {
            // Only delete if it has an original index (not a new binding)
            if let Some(original_index) = eb.original_index {
                self.keybindings_view_model
                    .pending_changes
                    .push(KeybindingChange::Delete(original_index));
            } else {
                // Remove the Add entry from pending_changes for new bindings
                self.keybindings_view_model.pending_changes.retain(|c| {
                    !matches!(c, KeybindingChange::Add(b) if b.combo() == eb.binding.combo())
                });
            }

            // Update selection if needed
            let count = self.keybindings_view_model.visible_count();
            if self.keybindings_view_model.selected_index >= count.saturating_sub(1) {
                self.keybindings_view_model.selected_index =
                    count.saturating_sub(2);
            }
        }
    }

    fn start_edit_keybinding(&mut self) {
        let filtered = self.keybindings_view_model.filtered_bindings();
        if let Some(eb) = filtered.get(self.keybindings_view_model.selected_index) {
            let original_index = eb.original_index.unwrap_or(0);
            self.keybindings_view_model.edit_mode =
                Some(EditMode::from_binding(original_index, &eb.binding));
            self.error = None;
        }
    }

    fn confirm_edit_keybinding(&mut self) {
        let edit_mode = match &self.keybindings_view_model.edit_mode {
            Some(em) => em.clone(),
            None => return,
        };

        // Validate and convert to keybinding
        let new_binding = match edit_mode.to_keybinding() {
            Some(kb) => kb,
            None => {
                self.error = Some("Invalid keybinding: key combo and action are required".to_string());
                return;
            }
        };

        // Add the change
        if edit_mode.is_new {
            self.keybindings_view_model
                .pending_changes
                .push(KeybindingChange::Add(new_binding));
        } else {
            self.keybindings_view_model
                .pending_changes
                .push(KeybindingChange::Modify {
                    index: edit_mode.original_index,
                    new: new_binding,
                });
        }

        // Exit edit mode
        self.keybindings_view_model.edit_mode = None;
        self.error = None;
    }

    fn preview_changes(&mut self) {
        if !self.view_model.has_pending_changes() {
            return;
        }

        let mut client = match NiriClient::connect() {
            Ok(c) => c,
            Err(e) => {
                self.error = Some(format!("Failed to connect: {e}"));
                return;
            }
        };

        for (name, pos) in &self.view_model.pending_changes {
            if let Err(e) = client.preview_position(name, *pos) {
                self.error = Some(format!("Preview failed for {name}: {e}"));
                return;
            }
        }
    }

    /// Handle keyboard input and return a message
    pub fn handle_input(&mut self) -> Result<Option<Message>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle F-keys for category switching (global)
                if let Some(category) = Category::from_function_key(key.code) {
                    return Ok(Some(Message::SwitchCategory(category)));
                }

                // Handle category-specific input
                let msg = match self.current_category {
                    Category::Outputs => self.handle_outputs_input(key.code, key.modifiers),
                    Category::Keybindings => self.handle_keybindings_input(key.code, key.modifiers),
                };
                return Ok(msg);
            }
        }
        Ok(None)
    }

    fn handle_outputs_input(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Message> {
        match (code, modifiers) {
            // Quit
            (KeyCode::Char('q'), _) => Some(Message::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Message::Quit),

            // Tab cycles between monitors
            (KeyCode::Tab, _) => Some(Message::SelectNextOutput),
            (KeyCode::BackTab, _) => Some(Message::SelectPrevOutput),

            // Snap positioning with Shift+HJKL (uppercase)
            (KeyCode::Char('H'), _) => Some(Message::SnapLeft),
            (KeyCode::Char('L'), _) => Some(Message::SnapRight),
            (KeyCode::Char('K'), _) => Some(Message::SnapAbove),
            (KeyCode::Char('J'), _) => Some(Message::SnapBelow),

            // hjkl for movement
            (KeyCode::Char('h'), _) => Some(Message::MoveOutput { dx: -10, dy: 0 }),
            (KeyCode::Char('j'), _) => Some(Message::MoveOutput { dx: 0, dy: 10 }),
            (KeyCode::Char('k'), _) => Some(Message::MoveOutput { dx: 0, dy: -10 }),
            (KeyCode::Char('l'), _) => Some(Message::MoveOutput { dx: 10, dy: 0 }),

            // Zoom (for large multi-monitor setups)
            (KeyCode::Char('+') | KeyCode::Char('='), _) => Some(Message::ZoomIn),
            (KeyCode::Char('-'), _) => Some(Message::ZoomOut),
            (KeyCode::Char('0'), _) => Some(Message::ResetView),

            // Normalize layout to origin
            (KeyCode::Char('n'), _) => Some(Message::Normalize),

            // Actions
            (KeyCode::Char('s'), _) => Some(Message::Save),
            (KeyCode::Char('r'), _) => Some(Message::Reload),
            (KeyCode::Char('p'), _) => Some(Message::PreviewChanges),
            (KeyCode::Esc, _) => Some(Message::RevertPreview),

            _ => None,
        }
    }

    fn handle_keybindings_input(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Option<Message> {
        // Handle edit mode input
        if self.keybindings_view_model.edit_mode.is_some() {
            return self.handle_edit_mode_input(code, modifiers);
        }

        // Handle search mode input
        if self.keybindings_view_model.search_mode {
            match code {
                KeyCode::Esc => {
                    return Some(Message::ClearSearch);
                }
                KeyCode::Enter => {
                    self.keybindings_view_model.search_mode = false;
                    return None;
                }
                KeyCode::Backspace => {
                    let mut query = self.keybindings_view_model.search_query.clone();
                    query.pop();
                    return Some(Message::UpdateSearch(query));
                }
                KeyCode::Char(c) => {
                    let mut query = self.keybindings_view_model.search_query.clone();
                    query.push(c);
                    return Some(Message::UpdateSearch(query));
                }
                _ => return None,
            }
        }

        match (code, modifiers) {
            // Quit
            (KeyCode::Char('q'), _) => Some(Message::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Message::Quit),

            // Navigation
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Some(Message::SelectNextKeybinding),
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Some(Message::SelectPrevKeybinding),

            // Search
            (KeyCode::Char('/'), _) => Some(Message::StartSearch),
            (KeyCode::Esc, _) => {
                if !self.keybindings_view_model.search_query.is_empty() {
                    Some(Message::ClearSearch)
                } else {
                    None
                }
            }

            // Actions
            (KeyCode::Enter, _) => Some(Message::StartEdit),
            (KeyCode::Char('a'), _) => Some(Message::AddKeybinding),
            (KeyCode::Char('d'), _) => Some(Message::DeleteKeybinding),
            (KeyCode::Char('s'), _) => Some(Message::Save),
            (KeyCode::Char('r'), _) => Some(Message::Reload),

            _ => None,
        }
    }

    fn handle_edit_mode_input(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Option<Message> {
        let edit_mode = match &mut self.keybindings_view_model.edit_mode {
            Some(em) => em,
            None => return None,
        };

        match code {
            KeyCode::Esc => Some(Message::CancelEdit),
            KeyCode::Enter => Some(Message::ConfirmEdit),
            KeyCode::Tab => {
                edit_mode.focused_field = edit_mode.focused_field.next();
                None
            }
            KeyCode::BackTab => {
                edit_mode.focused_field = edit_mode.focused_field.prev();
                None
            }
            // Up/Down arrows for field navigation
            KeyCode::Up => {
                edit_mode.focused_field = edit_mode.focused_field.prev();
                None
            }
            KeyCode::Down => {
                edit_mode.focused_field = edit_mode.focused_field.next();
                None
            }
            // Left/Right arrows for cursor movement in text fields, or action type cycling
            KeyCode::Left => {
                match edit_mode.focused_field {
                    EditField::KeyCombo | EditField::ActionValue => {
                        edit_mode.cursor_left();
                    }
                    EditField::ActionType => {
                        edit_mode.prev_action_type();
                    }
                    _ => {}
                }
                None
            }
            KeyCode::Right => {
                match edit_mode.focused_field {
                    EditField::KeyCombo | EditField::ActionValue => {
                        edit_mode.cursor_right();
                    }
                    EditField::ActionType => {
                        edit_mode.next_action_type();
                    }
                    _ => {}
                }
                None
            }
            // Home/End for cursor movement
            KeyCode::Home => {
                edit_mode.cursor_home();
                None
            }
            KeyCode::End => {
                edit_mode.cursor_end();
                None
            }
            KeyCode::Backspace => {
                edit_mode.delete_char();
                None
            }
            KeyCode::Char(' ') => {
                match edit_mode.focused_field {
                    EditField::Repeat => {
                        edit_mode.toggle_repeat();
                    }
                    EditField::AllowWhenLocked => {
                        edit_mode.toggle_allow_when_locked();
                    }
                    EditField::KeyCombo => {
                        // Don't add space to key combo
                    }
                    EditField::ActionType => {
                        // Space also cycles action type forward
                        edit_mode.next_action_type();
                    }
                    EditField::ActionValue => {
                        edit_mode.insert_char(' ');
                    }
                }
                None
            }
            KeyCode::Char(c) => {
                edit_mode.insert_char(c);
                None
            }
            _ => None,
        }
    }

    /// Render the UI
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Main layout: tab bar, body, footer
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tab bar
                Constraint::Min(10),   // Body
                Constraint::Length(2), // Status bar
            ])
            .split(size);

        // Tab bar
        let tab_bar = TabBarWidget::new(self.current_category);
        frame.render_widget(tab_bar, main_layout[0]);

        // Draw category-specific content
        match self.current_category {
            Category::Outputs => self.draw_outputs(frame, main_layout[1]),
            Category::Keybindings => self.draw_keybindings(frame, main_layout[1]),
        }

        // Status bar with category-specific keybinds
        let has_changes = match self.current_category {
            Category::Outputs => self.view_model.has_pending_changes(),
            Category::Keybindings => self.keybindings_view_model.has_pending_changes(),
        };
        let status = StatusBarWidget::new(
            has_changes,
            self.error.clone(),
            self.current_category.keybinds(),
        );
        frame.render_widget(status, main_layout[2]);
    }

    fn draw_outputs(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Body layout: left panel (list + info) and right panel (canvas)
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // Left panel
                Constraint::Min(30),    // Canvas
            ])
            .split(area);

        // Left panel: output list + info
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),     // Output list
                Constraint::Length(10), // Info panel
            ])
            .split(body_layout[0]);

        // Render widgets
        let output_list = OutputListWidget::new(&self.view_model, true);
        frame.render_widget(output_list, left_layout[0]);

        let output_info = OutputInfoWidget::new(&self.view_model);
        frame.render_widget(output_info, left_layout[1]);

        let canvas = MonitorCanvasWidget::new(&self.view_model, &self.viewport, true);
        frame.render_widget(canvas, body_layout[1]);
    }

    fn draw_keybindings(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Calculate visible height for scroll
        let inner_height = area.height.saturating_sub(2) as usize;
        self.keybindings_view_model.update_scroll(inner_height);

        // Body layout: list and detail panel
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(55), // Keybindings list
                Constraint::Percentage(45), // Detail panel
            ])
            .split(area);

        // Keybindings list
        let list = KeybindingsListWidget::new(&self.keybindings_view_model, true);
        frame.render_widget(list, body_layout[0]);

        // Detail panel with status
        let selected_eb = self.keybindings_view_model.selected_effective_binding();
        let (binding, status) = match selected_eb {
            Some(eb) => (Some(eb.binding), Some(eb.status)),
            None => (None, None),
        };
        let detail = KeybindingDetailWidget::with_status(binding, status);
        frame.render_widget(detail, body_layout[1]);

        // Edit dialog (renders on top if edit mode is active)
        if let Some(ref edit_mode) = self.keybindings_view_model.edit_mode {
            let edit_widget = KeybindingEditWidget::new(edit_mode);
            frame.render_widget(edit_widget, area);
        }
    }
}
