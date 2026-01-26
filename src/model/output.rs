use std::collections::HashMap;

/// Physical position in logical pixels
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Size in logical pixels
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Output mode (resolution and refresh rate)
#[derive(Debug, Clone, PartialEq)]
pub struct OutputMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f64, // Hz
    pub is_preferred: bool,
}

/// Transform for output rotation/flip
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputTransform {
    #[default]
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl OutputTransform {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputTransform::Normal => "normal",
            OutputTransform::Rotate90 => "90",
            OutputTransform::Rotate180 => "180",
            OutputTransform::Rotate270 => "270",
            OutputTransform::Flipped => "flipped",
            OutputTransform::Flipped90 => "flipped-90",
            OutputTransform::Flipped180 => "flipped-180",
            OutputTransform::Flipped270 => "flipped-270",
        }
    }

    pub fn from_niri(transform: &niri_ipc::Transform) -> Self {
        match transform {
            niri_ipc::Transform::Normal => OutputTransform::Normal,
            niri_ipc::Transform::_90 => OutputTransform::Rotate90,
            niri_ipc::Transform::_180 => OutputTransform::Rotate180,
            niri_ipc::Transform::_270 => OutputTransform::Rotate270,
            niri_ipc::Transform::Flipped => OutputTransform::Flipped,
            niri_ipc::Transform::Flipped90 => OutputTransform::Flipped90,
            niri_ipc::Transform::Flipped180 => OutputTransform::Flipped180,
            niri_ipc::Transform::Flipped270 => OutputTransform::Flipped270,
        }
    }
}

/// Complete state for a single output
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields are for future features
pub struct OutputState {
    pub name: String,
    pub modes: Vec<OutputMode>,
    pub current_mode_index: Option<usize>, // Index into modes
    pub scale: f64,
    pub transform: OutputTransform,
    pub position: Position,
    pub logical_size: Size,
    pub physical_size: Size,
    pub enabled: bool,
    pub connected: bool,
    pub configured: bool,
    pub make: String,
    pub model: String,
}

impl OutputState {
    pub fn current_mode(&self) -> Option<&OutputMode> {
        self.current_mode_index
            .and_then(|idx| self.modes.get(idx))
    }

    pub fn mode_string(&self) -> String {
        self.current_mode()
            .map(|m| format!("{}x{}@{:.2}Hz", m.width, m.height, m.refresh_rate))
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

/// View model for displaying outputs
#[derive(Debug, Clone, Default)]
pub struct OutputViewModel {
    pub outputs: Vec<OutputState>,
    pub selected_index: usize,
    pub pending_changes: HashMap<String, Position>,
}

impl OutputViewModel {
    pub fn selected_output(&self) -> Option<&OutputState> {
        self.outputs.get(self.selected_index)
    }

    #[allow(dead_code)] // For future features
    pub fn selected_output_mut(&mut self) -> Option<&mut OutputState> {
        self.outputs.get_mut(self.selected_index)
    }

    pub fn get_display_position(&self, name: &str) -> Option<Position> {
        self.pending_changes.get(name).copied().or_else(|| {
            self.outputs
                .iter()
                .find(|o| o.name == name)
                .map(|o| o.position)
        })
    }

    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    pub fn apply_pending_change(&mut self, name: &str, position: Position) {
        self.pending_changes.insert(name.to_string(), position);
    }

    pub fn clear_pending_changes(&mut self) {
        self.pending_changes.clear();
    }

    pub fn select_next(&mut self) {
        if !self.outputs.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.outputs.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.outputs.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.outputs.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}
