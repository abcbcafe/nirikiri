use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::time::Duration;

use crate::config::{get_configured_positions, load_config, write_positions};
use crate::ipc::NiriClient;
use crate::message::Message;
use crate::model::{ConfigDocument, OutputViewModel};
use crate::update::update_output;
use crate::view::{OutputInfoWidget, OutputListWidget, StatusBarWidget};
use crate::widgets::{CanvasViewport, MonitorCanvasWidget};

/// Focus states for different panels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    OutputList,
    Canvas,
}

impl FocusPanel {
    pub fn next(self) -> Self {
        match self {
            FocusPanel::OutputList => FocusPanel::Canvas,
            FocusPanel::Canvas => FocusPanel::OutputList,
        }
    }
}

/// Main application state
pub struct App {
    pub view_model: OutputViewModel,
    pub config: Option<ConfigDocument>,
    pub viewport: CanvasViewport,
    pub focus: FocusPanel,
    pub error: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            view_model: OutputViewModel::default(),
            config: None,
            viewport: CanvasViewport::default(),
            focus: FocusPanel::OutputList,
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
                    if let Some(output) = self.view_model.outputs.iter_mut().find(|o| &o.name == name) {
                        output.configured = true;
                    }
                }
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
            Message::FocusNext => {
                self.focus = self.focus.next();
            }
            Message::FocusPrev => {
                self.focus = self.focus.next(); // Same as next for 2 panels
            }
            Message::PanCanvas { dx, dy } => {
                self.viewport.pan(dx * 5, dy * 5);
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
            // Output-related messages
            msg => {
                update_output(&mut self.view_model, &msg);
            }
        }
    }

    fn save_config(&mut self) {
        if !self.view_model.has_pending_changes() {
            return;
        }

        if let Some(config) = &mut self.config {
            match write_positions(config, &self.view_model.pending_changes) {
                Ok(()) => {
                    // Apply pending changes to outputs
                    for (name, pos) in &self.view_model.pending_changes {
                        if let Some(output) = self.view_model.outputs.iter_mut().find(|o| &o.name == name) {
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
    pub fn handle_input(&self) -> Result<Option<Message>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let msg = match (key.code, key.modifiers, self.focus) {
                    // Quit
                    (KeyCode::Char('q'), _, _) => Some(Message::Quit),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL, _) => Some(Message::Quit),

                    // Focus switching
                    (KeyCode::Tab, _, _) => Some(Message::FocusNext),
                    (KeyCode::BackTab, _, _) => Some(Message::FocusPrev),

                    // Snap positioning with Shift+HJKL (uppercase)
                    (KeyCode::Char('H'), _, _) => Some(Message::SnapLeft),
                    (KeyCode::Char('L'), _, _) => Some(Message::SnapRight),
                    (KeyCode::Char('K'), _, _) => Some(Message::SnapAbove),
                    (KeyCode::Char('J'), _, _) => Some(Message::SnapBelow),

                    // Move selected monitor with shift+arrows (fine-grained movement)
                    (KeyCode::Up, mods, _) if mods.contains(KeyModifiers::SHIFT) => {
                        Some(Message::MoveOutput { dx: 0, dy: -10 })
                    }
                    (KeyCode::Down, mods, _) if mods.contains(KeyModifiers::SHIFT) => {
                        Some(Message::MoveOutput { dx: 0, dy: 10 })
                    }
                    (KeyCode::Left, mods, _) if mods.contains(KeyModifiers::SHIFT) => {
                        Some(Message::MoveOutput { dx: -10, dy: 0 })
                    }
                    (KeyCode::Right, mods, _) if mods.contains(KeyModifiers::SHIFT) => {
                        Some(Message::MoveOutput { dx: 10, dy: 0 })
                    }

                    // Snap with Ctrl+arrows (alternative)
                    (KeyCode::Up, mods, _) if mods.contains(KeyModifiers::CONTROL) => {
                        Some(Message::SnapAbove)
                    }
                    (KeyCode::Down, mods, _) if mods.contains(KeyModifiers::CONTROL) => {
                        Some(Message::SnapBelow)
                    }
                    (KeyCode::Left, mods, _) if mods.contains(KeyModifiers::CONTROL) => {
                        Some(Message::SnapLeft)
                    }
                    (KeyCode::Right, mods, _) if mods.contains(KeyModifiers::CONTROL) => {
                        Some(Message::SnapRight)
                    }

                    // Output list navigation (j/k or arrows select, h/l move horizontally)
                    (KeyCode::Up | KeyCode::Char('k'), _, FocusPanel::OutputList) => {
                        Some(Message::SelectPrevOutput)
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _, FocusPanel::OutputList) => {
                        Some(Message::SelectNextOutput)
                    }
                    (KeyCode::Left | KeyCode::Char('h'), _, FocusPanel::OutputList) => {
                        Some(Message::MoveOutput { dx: -10, dy: 0 })
                    }
                    (KeyCode::Right | KeyCode::Char('l'), _, FocusPanel::OutputList) => {
                        Some(Message::MoveOutput { dx: 10, dy: 0 })
                    }

                    // Canvas panning when canvas is focused
                    (KeyCode::Up | KeyCode::Char('k'), _, FocusPanel::Canvas) => {
                        Some(Message::PanCanvas { dx: 0, dy: -1 })
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _, FocusPanel::Canvas) => {
                        Some(Message::PanCanvas { dx: 0, dy: 1 })
                    }
                    (KeyCode::Left | KeyCode::Char('h'), _, FocusPanel::Canvas) => {
                        Some(Message::PanCanvas { dx: -1, dy: 0 })
                    }
                    (KeyCode::Right | KeyCode::Char('l'), _, FocusPanel::Canvas) => {
                        Some(Message::PanCanvas { dx: 1, dy: 0 })
                    }

                    // Zoom
                    (KeyCode::Char('+') | KeyCode::Char('='), _, _) => Some(Message::ZoomIn),
                    (KeyCode::Char('-'), _, _) => Some(Message::ZoomOut),
                    (KeyCode::Char('0'), _, _) => Some(Message::ResetView),

                    // Normalize layout to origin
                    (KeyCode::Char('n'), _, _) => Some(Message::Normalize),

                    // Actions
                    (KeyCode::Char('s'), _, _) => Some(Message::Save),
                    (KeyCode::Char('r'), _, _) => Some(Message::Reload),
                    (KeyCode::Char('p'), _, _) => Some(Message::PreviewChanges),
                    (KeyCode::Esc, _, _) => Some(Message::RevertPreview),

                    _ => None,
                };
                return Ok(msg);
            }
        }
        Ok(None)
    }

    /// Render the UI
    pub fn draw(&self, frame: &mut Frame) {
        let size = frame.area();

        // Main layout: header, body, footer
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title
                Constraint::Min(10),    // Body
                Constraint::Length(2),  // Status bar
            ])
            .split(size);

        // Title
        let title = ratatui::widgets::Paragraph::new(" niritui - niri Output Configuration")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(title, main_layout[0]);

        // Body layout: left panel (list + info) and right panel (canvas)
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // Left panel
                Constraint::Min(30),    // Canvas
            ])
            .split(main_layout[1]);

        // Left panel: output list + info
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),     // Output list
                Constraint::Length(10), // Info panel
            ])
            .split(body_layout[0]);

        // Render widgets
        let output_list = OutputListWidget::new(&self.view_model, self.focus == FocusPanel::OutputList);
        frame.render_widget(output_list, left_layout[0]);

        let output_info = OutputInfoWidget::new(&self.view_model);
        frame.render_widget(output_info, left_layout[1]);

        let canvas = MonitorCanvasWidget::new(&self.view_model, &self.viewport, self.focus == FocusPanel::Canvas);
        frame.render_widget(canvas, body_layout[1]);

        // Status bar
        let status = StatusBarWidget::new(
            self.view_model.has_pending_changes(),
            self.error.clone(),
        );
        frame.render_widget(status, main_layout[2]);
    }
}
