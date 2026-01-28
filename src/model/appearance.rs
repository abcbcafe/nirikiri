use std::fmt;

/// A color value that can be either solid or a gradient
#[derive(Debug, Clone, PartialEq)]
pub enum ColorValue {
    Solid(String),
    Gradient {
        from: String,
        to: String,
        angle: Option<i32>,
        relative_to: Option<String>,
        color_space: Option<String>,
    },
}

impl Default for ColorValue {
    fn default() -> Self {
        ColorValue::Solid("#ffffff".to_string())
    }
}

impl fmt::Display for ColorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorValue::Solid(color) => write!(f, "{color}"),
            ColorValue::Gradient { from, to, angle, relative_to, color_space } => {
                let mut parts = vec![format!("from={from}"), format!("to={to}")];
                if let Some(a) = angle {
                    parts.push(format!("angle={a}"));
                }
                if let Some(r) = relative_to {
                    parts.push(format!("relative-to={r}"));
                }
                if let Some(c) = color_space {
                    parts.push(format!("in={c}"));
                }
                write!(f, "gradient({})", parts.join(" "))
            }
        }
    }
}

/// When to center a focused column
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CenterFocusedColumn {
    #[default]
    Never,
    Always,
    OnOverflow,
}

impl CenterFocusedColumn {
    pub fn as_str(&self) -> &'static str {
        match self {
            CenterFocusedColumn::Never => "never",
            CenterFocusedColumn::Always => "always",
            CenterFocusedColumn::OnOverflow => "on-overflow",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "never" => Some(CenterFocusedColumn::Never),
            "always" => Some(CenterFocusedColumn::Always),
            "on-overflow" => Some(CenterFocusedColumn::OnOverflow),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            CenterFocusedColumn::Never => CenterFocusedColumn::Always,
            CenterFocusedColumn::Always => CenterFocusedColumn::OnOverflow,
            CenterFocusedColumn::OnOverflow => CenterFocusedColumn::Never,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            CenterFocusedColumn::Never => CenterFocusedColumn::OnOverflow,
            CenterFocusedColumn::Always => CenterFocusedColumn::Never,
            CenterFocusedColumn::OnOverflow => CenterFocusedColumn::Always,
        }
    }
}

impl fmt::Display for CenterFocusedColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Focus ring settings
#[derive(Debug, Clone, PartialEq)]
pub struct FocusRingSettings {
    pub off: bool,
    pub width: i32,
    pub active_color: ColorValue,
    pub inactive_color: ColorValue,
    pub active_gradient: Option<ColorValue>,
    pub inactive_gradient: Option<ColorValue>,
}

impl Default for FocusRingSettings {
    fn default() -> Self {
        Self {
            off: false,
            width: 4,
            active_color: ColorValue::Solid("#7fc8ff".to_string()),
            inactive_color: ColorValue::Solid("#505050".to_string()),
            active_gradient: None,
            inactive_gradient: None,
        }
    }
}

/// Border settings
#[derive(Debug, Clone, PartialEq)]
pub struct BorderSettings {
    pub off: bool,
    pub width: i32,
    pub active_color: ColorValue,
    pub inactive_color: ColorValue,
    pub urgent_color: Option<ColorValue>,
    pub active_gradient: Option<ColorValue>,
    pub inactive_gradient: Option<ColorValue>,
}

impl Default for BorderSettings {
    fn default() -> Self {
        Self {
            off: true,
            width: 4,
            active_color: ColorValue::Solid("#ffc87f".to_string()),
            inactive_color: ColorValue::Solid("#505050".to_string()),
            urgent_color: Some(ColorValue::Solid("#9b0000".to_string())),
            active_gradient: None,
            inactive_gradient: None,
        }
    }
}

/// Shadow settings
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowSettings {
    pub on: bool,
    pub draw_behind_window: bool,
    pub softness: i32,
    pub spread: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub color: ColorValue,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            on: false,
            draw_behind_window: false,
            softness: 30,
            spread: 5,
            offset_x: 0,
            offset_y: 5,
            color: ColorValue::Solid("#0007".to_string()),
        }
    }
}

/// Struts settings (outer gaps)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StrutsSettings {
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub top: Option<i32>,
    pub bottom: Option<i32>,
}

/// All appearance settings from the layout block
#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceSettings {
    pub gaps: i32,
    pub center_focused_column: CenterFocusedColumn,
    pub focus_ring: FocusRingSettings,
    pub border: BorderSettings,
    pub shadow: ShadowSettings,
    pub struts: StrutsSettings,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            gaps: 16,
            center_focused_column: CenterFocusedColumn::default(),
            focus_ring: FocusRingSettings::default(),
            border: BorderSettings::default(),
            shadow: ShadowSettings::default(),
            struts: StrutsSettings::default(),
        }
    }
}

/// Sections in the appearance settings list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceSection {
    General,
    FocusRing,
    Border,
    Shadow,
    Struts,
}

impl AppearanceSection {
    pub fn all() -> &'static [AppearanceSection] {
        &[
            AppearanceSection::General,
            AppearanceSection::FocusRing,
            AppearanceSection::Border,
            AppearanceSection::Shadow,
            AppearanceSection::Struts,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AppearanceSection::General => "General",
            AppearanceSection::FocusRing => "Focus Ring",
            AppearanceSection::Border => "Border",
            AppearanceSection::Shadow => "Shadow",
            AppearanceSection::Struts => "Struts",
        }
    }

    pub fn fields(&self) -> &'static [AppearanceField] {
        match self {
            AppearanceSection::General => &[
                AppearanceField::Gaps,
                AppearanceField::CenterFocusedColumn,
            ],
            AppearanceSection::FocusRing => &[
                AppearanceField::FocusRingOff,
                AppearanceField::FocusRingWidth,
                AppearanceField::FocusRingActiveColor,
                AppearanceField::FocusRingInactiveColor,
            ],
            AppearanceSection::Border => &[
                AppearanceField::BorderOff,
                AppearanceField::BorderWidth,
                AppearanceField::BorderActiveColor,
                AppearanceField::BorderInactiveColor,
                AppearanceField::BorderUrgentColor,
            ],
            AppearanceSection::Shadow => &[
                AppearanceField::ShadowOn,
                AppearanceField::ShadowDrawBehindWindow,
                AppearanceField::ShadowSoftness,
                AppearanceField::ShadowSpread,
                AppearanceField::ShadowOffsetX,
                AppearanceField::ShadowOffsetY,
                AppearanceField::ShadowColor,
            ],
            AppearanceSection::Struts => &[
                AppearanceField::StrutsLeft,
                AppearanceField::StrutsRight,
                AppearanceField::StrutsTop,
                AppearanceField::StrutsBottom,
            ],
        }
    }
}

/// Individual fields that can be edited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceField {
    // General
    Gaps,
    CenterFocusedColumn,
    // Focus Ring
    FocusRingOff,
    FocusRingWidth,
    FocusRingActiveColor,
    FocusRingInactiveColor,
    // Border
    BorderOff,
    BorderWidth,
    BorderActiveColor,
    BorderInactiveColor,
    BorderUrgentColor,
    // Shadow
    ShadowOn,
    ShadowDrawBehindWindow,
    ShadowSoftness,
    ShadowSpread,
    ShadowOffsetX,
    ShadowOffsetY,
    ShadowColor,
    // Struts
    StrutsLeft,
    StrutsRight,
    StrutsTop,
    StrutsBottom,
}

impl AppearanceField {
    pub fn name(&self) -> &'static str {
        match self {
            AppearanceField::Gaps => "gaps",
            AppearanceField::CenterFocusedColumn => "center-focused-column",
            AppearanceField::FocusRingOff => "off",
            AppearanceField::FocusRingWidth => "width",
            AppearanceField::FocusRingActiveColor => "active-color",
            AppearanceField::FocusRingInactiveColor => "inactive-color",
            AppearanceField::BorderOff => "off",
            AppearanceField::BorderWidth => "width",
            AppearanceField::BorderActiveColor => "active-color",
            AppearanceField::BorderInactiveColor => "inactive-color",
            AppearanceField::BorderUrgentColor => "urgent-color",
            AppearanceField::ShadowOn => "on",
            AppearanceField::ShadowDrawBehindWindow => "draw-behind-window",
            AppearanceField::ShadowSoftness => "softness",
            AppearanceField::ShadowSpread => "spread",
            AppearanceField::ShadowOffsetX => "offset x",
            AppearanceField::ShadowOffsetY => "offset y",
            AppearanceField::ShadowColor => "color",
            AppearanceField::StrutsLeft => "left",
            AppearanceField::StrutsRight => "right",
            AppearanceField::StrutsTop => "top",
            AppearanceField::StrutsBottom => "bottom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AppearanceField::Gaps => "Gap size between windows in logical pixels",
            AppearanceField::CenterFocusedColumn => "When to center the focused column: never, always, or on-overflow",
            AppearanceField::FocusRingOff => "Disable the focus ring entirely",
            AppearanceField::FocusRingWidth => "Width of the focus ring in logical pixels",
            AppearanceField::FocusRingActiveColor => "Color of the focus ring on the active monitor",
            AppearanceField::FocusRingInactiveColor => "Color of the focus ring on inactive monitors",
            AppearanceField::BorderOff => "Disable/enable the border (off by default)",
            AppearanceField::BorderWidth => "Width of the border in logical pixels",
            AppearanceField::BorderActiveColor => "Color of the border on the active window",
            AppearanceField::BorderInactiveColor => "Color of the border on inactive windows",
            AppearanceField::BorderUrgentColor => "Color of the border for urgent windows",
            AppearanceField::ShadowOn => "Enable drop shadows for windows",
            AppearanceField::ShadowDrawBehindWindow => "Draw shadow behind the window (fixes CSD corners)",
            AppearanceField::ShadowSoftness => "Shadow blur radius in logical pixels",
            AppearanceField::ShadowSpread => "Shadow expansion in logical pixels",
            AppearanceField::ShadowOffsetX => "Horizontal shadow offset in logical pixels",
            AppearanceField::ShadowOffsetY => "Vertical shadow offset in logical pixels",
            AppearanceField::ShadowColor => "Shadow color (supports alpha, e.g. #0007)",
            AppearanceField::StrutsLeft => "Left strut (outer gap) in logical pixels",
            AppearanceField::StrutsRight => "Right strut (outer gap) in logical pixels",
            AppearanceField::StrutsTop => "Top strut (outer gap) in logical pixels",
            AppearanceField::StrutsBottom => "Bottom strut (outer gap) in logical pixels",
        }
    }

    pub fn section(&self) -> AppearanceSection {
        match self {
            AppearanceField::Gaps | AppearanceField::CenterFocusedColumn => AppearanceSection::General,
            AppearanceField::FocusRingOff
            | AppearanceField::FocusRingWidth
            | AppearanceField::FocusRingActiveColor
            | AppearanceField::FocusRingInactiveColor => AppearanceSection::FocusRing,
            AppearanceField::BorderOff
            | AppearanceField::BorderWidth
            | AppearanceField::BorderActiveColor
            | AppearanceField::BorderInactiveColor
            | AppearanceField::BorderUrgentColor => AppearanceSection::Border,
            AppearanceField::ShadowOn
            | AppearanceField::ShadowDrawBehindWindow
            | AppearanceField::ShadowSoftness
            | AppearanceField::ShadowSpread
            | AppearanceField::ShadowOffsetX
            | AppearanceField::ShadowOffsetY
            | AppearanceField::ShadowColor => AppearanceSection::Shadow,
            AppearanceField::StrutsLeft
            | AppearanceField::StrutsRight
            | AppearanceField::StrutsTop
            | AppearanceField::StrutsBottom => AppearanceSection::Struts,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(
            self,
            AppearanceField::FocusRingOff
                | AppearanceField::BorderOff
                | AppearanceField::ShadowOn
                | AppearanceField::ShadowDrawBehindWindow
        )
    }

    /// Returns true for boolean fields where `true` means "disabled/off"
    /// (i.e., the display should be inverted)
    pub fn is_off_semantic(&self) -> bool {
        matches!(
            self,
            AppearanceField::FocusRingOff | AppearanceField::BorderOff
        )
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, AppearanceField::CenterFocusedColumn)
    }

    pub fn is_color(&self) -> bool {
        matches!(
            self,
            AppearanceField::FocusRingActiveColor
                | AppearanceField::FocusRingInactiveColor
                | AppearanceField::BorderActiveColor
                | AppearanceField::BorderInactiveColor
                | AppearanceField::BorderUrgentColor
                | AppearanceField::ShadowColor
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            AppearanceField::Gaps
                | AppearanceField::FocusRingWidth
                | AppearanceField::BorderWidth
                | AppearanceField::ShadowSoftness
                | AppearanceField::ShadowSpread
                | AppearanceField::ShadowOffsetX
                | AppearanceField::ShadowOffsetY
                | AppearanceField::StrutsLeft
                | AppearanceField::StrutsRight
                | AppearanceField::StrutsTop
                | AppearanceField::StrutsBottom
        )
    }
}

/// Type of value being edited
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Boolean(bool),
    Integer(i32),
    OptionalInteger(Option<i32>),
    String(String),
    Enum(CenterFocusedColumn),
    Color(ColorValue),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Boolean(b) => write!(f, "{}", if *b { "on" } else { "off" }),
            FieldValue::Integer(n) => write!(f, "{n}"),
            FieldValue::OptionalInteger(opt) => match opt {
                Some(n) => write!(f, "{n}"),
                None => write!(f, "(not set)"),
            },
            FieldValue::String(s) => write!(f, "{s}"),
            FieldValue::Enum(e) => write!(f, "{e}"),
            FieldValue::Color(c) => write!(f, "{c}"),
        }
    }
}

/// A single setting change
#[derive(Debug, Clone)]
#[allow(dead_code)] // value field is stored for potential future use (e.g., undo)
pub struct AppearanceChange {
    pub field: AppearanceField,
    pub value: FieldValue,
}

/// Which field is focused in a color/gradient editor
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorEditField {
    #[default]
    ColorType,  // Solid vs Gradient selector
    SolidColor,
    GradientFrom,
    GradientTo,
    GradientAngle,
    GradientRelativeTo,
}

impl ColorEditField {
    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            ColorEditField::ColorType => ColorEditField::SolidColor,
            ColorEditField::SolidColor => ColorEditField::ColorType,
            ColorEditField::GradientFrom => ColorEditField::GradientTo,
            ColorEditField::GradientTo => ColorEditField::GradientAngle,
            ColorEditField::GradientAngle => ColorEditField::GradientRelativeTo,
            ColorEditField::GradientRelativeTo => ColorEditField::GradientFrom,
        }
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        match self {
            ColorEditField::ColorType => ColorEditField::SolidColor,
            ColorEditField::SolidColor => ColorEditField::ColorType,
            ColorEditField::GradientFrom => ColorEditField::GradientRelativeTo,
            ColorEditField::GradientTo => ColorEditField::GradientFrom,
            ColorEditField::GradientAngle => ColorEditField::GradientTo,
            ColorEditField::GradientRelativeTo => ColorEditField::GradientAngle,
        }
    }

    pub fn next_for_mode(&self, is_gradient: bool) -> Self {
        if is_gradient {
            match self {
                ColorEditField::ColorType => ColorEditField::GradientFrom,
                ColorEditField::GradientFrom => ColorEditField::GradientTo,
                ColorEditField::GradientTo => ColorEditField::GradientAngle,
                ColorEditField::GradientAngle => ColorEditField::GradientRelativeTo,
                ColorEditField::GradientRelativeTo => ColorEditField::ColorType,
                _ => ColorEditField::GradientFrom,
            }
        } else {
            match self {
                ColorEditField::ColorType => ColorEditField::SolidColor,
                ColorEditField::SolidColor => ColorEditField::ColorType,
                _ => ColorEditField::SolidColor,
            }
        }
    }

    pub fn prev_for_mode(&self, is_gradient: bool) -> Self {
        if is_gradient {
            match self {
                ColorEditField::ColorType => ColorEditField::GradientRelativeTo,
                ColorEditField::GradientFrom => ColorEditField::ColorType,
                ColorEditField::GradientTo => ColorEditField::GradientFrom,
                ColorEditField::GradientAngle => ColorEditField::GradientTo,
                ColorEditField::GradientRelativeTo => ColorEditField::GradientAngle,
                _ => ColorEditField::GradientFrom,
            }
        } else {
            match self {
                ColorEditField::ColorType => ColorEditField::SolidColor,
                ColorEditField::SolidColor => ColorEditField::ColorType,
                _ => ColorEditField::SolidColor,
            }
        }
    }
}

/// State for editing a color (solid or gradient)
#[derive(Debug, Clone)]
pub struct ColorEditState {
    pub is_gradient: bool,
    pub focused_field: ColorEditField,
    // Solid color
    pub solid_color: String,
    pub solid_cursor: usize,
    // Gradient fields
    pub gradient_from: String,
    pub gradient_from_cursor: usize,
    pub gradient_to: String,
    pub gradient_to_cursor: usize,
    pub gradient_angle: String,
    pub gradient_angle_cursor: usize,
    pub gradient_relative_to: String, // "window" or "workspace-view"
}

impl ColorEditState {
    pub fn from_solid(color: &str) -> Self {
        let len = color.len();
        Self {
            is_gradient: false,
            focused_field: ColorEditField::SolidColor,
            solid_color: color.to_string(),
            solid_cursor: len,
            gradient_from: String::new(),
            gradient_from_cursor: 0,
            gradient_to: String::new(),
            gradient_to_cursor: 0,
            gradient_angle: String::new(),
            gradient_angle_cursor: 0,
            gradient_relative_to: "window".to_string(),
        }
    }

    pub fn from_gradient(from: &str, to: &str, angle: Option<i32>, relative_to: Option<&str>) -> Self {
        let angle_str = angle.map(|a| a.to_string()).unwrap_or_default();
        let angle_cursor = angle_str.len();
        Self {
            is_gradient: true,
            focused_field: ColorEditField::GradientFrom,
            solid_color: String::new(),
            solid_cursor: 0,
            gradient_from: from.to_string(),
            gradient_from_cursor: from.len(),
            gradient_to: to.to_string(),
            gradient_to_cursor: to.len(),
            gradient_angle: angle_str,
            gradient_angle_cursor: angle_cursor,
            gradient_relative_to: relative_to.unwrap_or("window").to_string(),
        }
    }

    pub fn toggle_type(&mut self) {
        self.is_gradient = !self.is_gradient;
        if self.is_gradient {
            self.focused_field = ColorEditField::GradientFrom;
            // Copy solid color to gradient from if empty
            if self.gradient_from.is_empty() && !self.solid_color.is_empty() {
                self.gradient_from = self.solid_color.clone();
                self.gradient_from_cursor = self.gradient_from.len();
            }
        } else {
            self.focused_field = ColorEditField::SolidColor;
            // Copy gradient from to solid if empty
            if self.solid_color.is_empty() && !self.gradient_from.is_empty() {
                self.solid_color = self.gradient_from.clone();
                self.solid_cursor = self.solid_color.len();
            }
        }
    }

    pub fn cycle_relative_to(&mut self) {
        self.gradient_relative_to = if self.gradient_relative_to == "window" {
            "workspace-view".to_string()
        } else {
            "window".to_string()
        };
    }

    fn current_text_mut(&mut self) -> Option<(&mut String, &mut usize)> {
        match self.focused_field {
            ColorEditField::SolidColor => Some((&mut self.solid_color, &mut self.solid_cursor)),
            ColorEditField::GradientFrom => Some((&mut self.gradient_from, &mut self.gradient_from_cursor)),
            ColorEditField::GradientTo => Some((&mut self.gradient_to, &mut self.gradient_to_cursor)),
            ColorEditField::GradientAngle => Some((&mut self.gradient_angle, &mut self.gradient_angle_cursor)),
            _ => None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some((text, cursor)) = self.current_text_mut() {
            text.insert(*cursor, c);
            *cursor += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if let Some((text, cursor)) = self.current_text_mut() {
            if *cursor > 0 {
                *cursor -= 1;
                text.remove(*cursor);
            }
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some((_, cursor)) = self.current_text_mut() {
            *cursor = cursor.saturating_sub(1);
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some((text, cursor)) = self.current_text_mut() {
            *cursor = (*cursor + 1).min(text.len());
        }
    }

    pub fn to_color_value(&self) -> Option<ColorValue> {
        if self.is_gradient {
            if self.gradient_from.is_empty() || self.gradient_to.is_empty() {
                return None;
            }
            let angle = self.gradient_angle.parse::<i32>().ok();
            let relative_to = if self.gradient_relative_to == "window" {
                None
            } else {
                Some(self.gradient_relative_to.clone())
            };
            Some(ColorValue::Gradient {
                from: self.gradient_from.clone(),
                to: self.gradient_to.clone(),
                angle,
                relative_to,
                color_space: None, // Could add this later
            })
        } else {
            if self.solid_color.is_empty() {
                return None;
            }
            Some(ColorValue::Solid(self.solid_color.clone()))
        }
    }
}

/// State for editing an appearance setting
#[derive(Debug, Clone)]
pub struct AppearanceEditMode {
    pub field: AppearanceField,
    // For simple values (integers, strings)
    pub value: String,
    pub cursor: usize,
    // For color editing
    pub color_state: Option<ColorEditState>,
}

impl AppearanceEditMode {
    pub fn new(field: AppearanceField, initial_value: &str) -> Self {
        let cursor = initial_value.len();
        Self {
            field,
            value: initial_value.to_string(),
            cursor,
            color_state: None,
        }
    }

    pub fn new_color(field: AppearanceField, color: &ColorValue) -> Self {
        let color_state = match color {
            ColorValue::Solid(c) => ColorEditState::from_solid(c),
            ColorValue::Gradient { from, to, angle, relative_to, .. } => {
                ColorEditState::from_gradient(from, to, *angle, relative_to.as_deref())
            }
        };
        Self {
            field,
            value: String::new(),
            cursor: 0,
            color_state: Some(color_state),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(ref mut cs) = self.color_state {
            cs.insert_char(c);
        } else {
            self.value.insert(self.cursor, c);
            self.cursor += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(ref mut cs) = self.color_state {
            cs.delete_char();
        } else if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some(ref mut cs) = self.color_state {
            cs.cursor_left();
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(ref mut cs) = self.color_state {
            cs.cursor_right();
        } else {
            self.cursor = (self.cursor + 1).min(self.value.len());
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
        if let Some(ref mut cs) = self.color_state {
            match cs.focused_field {
                ColorEditField::SolidColor => cs.solid_cursor = 0,
                ColorEditField::GradientFrom => cs.gradient_from_cursor = 0,
                ColorEditField::GradientTo => cs.gradient_to_cursor = 0,
                ColorEditField::GradientAngle => cs.gradient_angle_cursor = 0,
                _ => {}
            }
        }
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.value.len();
        if let Some(ref mut cs) = self.color_state {
            match cs.focused_field {
                ColorEditField::SolidColor => cs.solid_cursor = cs.solid_color.len(),
                ColorEditField::GradientFrom => cs.gradient_from_cursor = cs.gradient_from.len(),
                ColorEditField::GradientTo => cs.gradient_to_cursor = cs.gradient_to.len(),
                ColorEditField::GradientAngle => cs.gradient_angle_cursor = cs.gradient_angle.len(),
                _ => {}
            }
        }
    }
}

/// A list item in the appearance settings list
#[derive(Debug, Clone)]
pub enum AppearanceListItem {
    SectionHeader(AppearanceSection),
    Field(AppearanceField),
}

/// View model for the appearance category
#[derive(Debug, Default)]
pub struct AppearanceViewModel {
    pub settings: AppearanceSettings,
    pub original_settings: AppearanceSettings,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub collapsed_sections: std::collections::HashSet<AppearanceSection>,
    pub pending_changes: Vec<AppearanceChange>,
    pub edit_mode: Option<AppearanceEditMode>,
}

impl AppearanceViewModel {
    pub fn new(settings: AppearanceSettings) -> Self {
        Self {
            original_settings: settings.clone(),
            settings,
            selected_index: 0,
            scroll_offset: 0,
            collapsed_sections: std::collections::HashSet::new(),
            pending_changes: Vec::new(),
            edit_mode: None,
        }
    }

    /// Get the list of visible items (respecting collapsed sections)
    pub fn visible_items(&self) -> Vec<AppearanceListItem> {
        let mut items = Vec::new();
        for section in AppearanceSection::all() {
            items.push(AppearanceListItem::SectionHeader(*section));
            if !self.collapsed_sections.contains(section) {
                for field in section.fields() {
                    items.push(AppearanceListItem::Field(*field));
                }
            }
        }
        items
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<AppearanceListItem> {
        self.visible_items().get(self.selected_index).cloned()
    }

    /// Select next item
    pub fn select_next(&mut self) {
        let count = self.visible_items().len();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        let count = self.visible_items().len();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Toggle section collapsed state
    pub fn toggle_section(&mut self, section: AppearanceSection) {
        if self.collapsed_sections.contains(&section) {
            self.collapsed_sections.remove(&section);
        } else {
            self.collapsed_sections.insert(section);
        }
    }

    /// Toggle the selected section if it's a section header
    pub fn toggle_selected_section(&mut self) {
        if let Some(AppearanceListItem::SectionHeader(section)) = self.selected_item() {
            self.toggle_section(section);
        }
    }

    /// Update scroll offset for visible area
    pub fn update_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    /// Check if there are pending changes
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    /// Get the current value for a field
    pub fn get_field_value(&self, field: AppearanceField) -> FieldValue {
        match field {
            AppearanceField::Gaps => FieldValue::Integer(self.settings.gaps),
            AppearanceField::CenterFocusedColumn => FieldValue::Enum(self.settings.center_focused_column),
            AppearanceField::FocusRingOff => FieldValue::Boolean(self.settings.focus_ring.off),
            AppearanceField::FocusRingWidth => FieldValue::Integer(self.settings.focus_ring.width),
            AppearanceField::FocusRingActiveColor => FieldValue::Color(self.settings.focus_ring.active_color.clone()),
            AppearanceField::FocusRingInactiveColor => FieldValue::Color(self.settings.focus_ring.inactive_color.clone()),
            AppearanceField::BorderOff => FieldValue::Boolean(self.settings.border.off),
            AppearanceField::BorderWidth => FieldValue::Integer(self.settings.border.width),
            AppearanceField::BorderActiveColor => FieldValue::Color(self.settings.border.active_color.clone()),
            AppearanceField::BorderInactiveColor => FieldValue::Color(self.settings.border.inactive_color.clone()),
            AppearanceField::BorderUrgentColor => {
                match &self.settings.border.urgent_color {
                    Some(c) => FieldValue::Color(c.clone()),
                    None => FieldValue::String("(not set)".to_string()),
                }
            }
            AppearanceField::ShadowOn => FieldValue::Boolean(self.settings.shadow.on),
            AppearanceField::ShadowDrawBehindWindow => FieldValue::Boolean(self.settings.shadow.draw_behind_window),
            AppearanceField::ShadowSoftness => FieldValue::Integer(self.settings.shadow.softness),
            AppearanceField::ShadowSpread => FieldValue::Integer(self.settings.shadow.spread),
            AppearanceField::ShadowOffsetX => FieldValue::Integer(self.settings.shadow.offset_x),
            AppearanceField::ShadowOffsetY => FieldValue::Integer(self.settings.shadow.offset_y),
            AppearanceField::ShadowColor => FieldValue::Color(self.settings.shadow.color.clone()),
            AppearanceField::StrutsLeft => FieldValue::OptionalInteger(self.settings.struts.left),
            AppearanceField::StrutsRight => FieldValue::OptionalInteger(self.settings.struts.right),
            AppearanceField::StrutsTop => FieldValue::OptionalInteger(self.settings.struts.top),
            AppearanceField::StrutsBottom => FieldValue::OptionalInteger(self.settings.struts.bottom),
        }
    }

    /// Set a field value and track the change
    pub fn set_field_value(&mut self, field: AppearanceField, value: FieldValue) {
        match (field, &value) {
            (AppearanceField::Gaps, FieldValue::Integer(n)) => self.settings.gaps = *n,
            (AppearanceField::CenterFocusedColumn, FieldValue::Enum(e)) => self.settings.center_focused_column = *e,
            (AppearanceField::FocusRingOff, FieldValue::Boolean(b)) => self.settings.focus_ring.off = *b,
            (AppearanceField::FocusRingWidth, FieldValue::Integer(n)) => self.settings.focus_ring.width = *n,
            (AppearanceField::FocusRingActiveColor, FieldValue::Color(c)) => self.settings.focus_ring.active_color = c.clone(),
            (AppearanceField::FocusRingInactiveColor, FieldValue::Color(c)) => self.settings.focus_ring.inactive_color = c.clone(),
            (AppearanceField::BorderOff, FieldValue::Boolean(b)) => self.settings.border.off = *b,
            (AppearanceField::BorderWidth, FieldValue::Integer(n)) => self.settings.border.width = *n,
            (AppearanceField::BorderActiveColor, FieldValue::Color(c)) => self.settings.border.active_color = c.clone(),
            (AppearanceField::BorderInactiveColor, FieldValue::Color(c)) => self.settings.border.inactive_color = c.clone(),
            (AppearanceField::BorderUrgentColor, FieldValue::Color(c)) => self.settings.border.urgent_color = Some(c.clone()),
            (AppearanceField::ShadowOn, FieldValue::Boolean(b)) => self.settings.shadow.on = *b,
            (AppearanceField::ShadowDrawBehindWindow, FieldValue::Boolean(b)) => self.settings.shadow.draw_behind_window = *b,
            (AppearanceField::ShadowSoftness, FieldValue::Integer(n)) => self.settings.shadow.softness = *n,
            (AppearanceField::ShadowSpread, FieldValue::Integer(n)) => self.settings.shadow.spread = *n,
            (AppearanceField::ShadowOffsetX, FieldValue::Integer(n)) => self.settings.shadow.offset_x = *n,
            (AppearanceField::ShadowOffsetY, FieldValue::Integer(n)) => self.settings.shadow.offset_y = *n,
            (AppearanceField::ShadowColor, FieldValue::Color(c)) => self.settings.shadow.color = c.clone(),
            (AppearanceField::StrutsLeft, FieldValue::OptionalInteger(opt)) => self.settings.struts.left = *opt,
            (AppearanceField::StrutsRight, FieldValue::OptionalInteger(opt)) => self.settings.struts.right = *opt,
            (AppearanceField::StrutsTop, FieldValue::OptionalInteger(opt)) => self.settings.struts.top = *opt,
            (AppearanceField::StrutsBottom, FieldValue::OptionalInteger(opt)) => self.settings.struts.bottom = *opt,
            _ => return,
        }

        // Remove any existing change for this field and add the new one
        self.pending_changes.retain(|c| c.field != field);
        self.pending_changes.push(AppearanceChange { field, value });
    }

    /// Check if a field has been modified
    pub fn is_field_modified(&self, field: AppearanceField) -> bool {
        self.pending_changes.iter().any(|c| c.field == field)
    }

    /// Toggle a boolean field
    pub fn toggle_boolean(&mut self, field: AppearanceField) {
        if let FieldValue::Boolean(current) = self.get_field_value(field) {
            self.set_field_value(field, FieldValue::Boolean(!current));
        }
    }

    /// Increment an integer field
    pub fn increment_field(&mut self, field: AppearanceField, amount: i32) {
        match self.get_field_value(field) {
            FieldValue::Integer(n) => {
                self.set_field_value(field, FieldValue::Integer(n + amount));
            }
            FieldValue::OptionalInteger(opt) => {
                let new_val = opt.unwrap_or(0) + amount;
                self.set_field_value(field, FieldValue::OptionalInteger(Some(new_val)));
            }
            _ => {}
        }
    }

    /// Cycle an enum field
    pub fn cycle_enum(&mut self, field: AppearanceField, forward: bool) {
        if let FieldValue::Enum(current) = self.get_field_value(field) {
            let new_val = if forward { current.next() } else { current.prev() };
            self.set_field_value(field, FieldValue::Enum(new_val));
        }
    }

    /// Clear pending changes and reset to original
    pub fn reset_changes(&mut self) {
        self.settings = self.original_settings.clone();
        self.pending_changes.clear();
    }

    /// Apply pending changes to original (after save)
    pub fn apply_changes(&mut self) {
        self.original_settings = self.settings.clone();
        self.pending_changes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_value_display() {
        assert_eq!(ColorValue::Solid("#ff0000".to_string()).to_string(), "#ff0000");

        let gradient = ColorValue::Gradient {
            from: "#ff0000".to_string(),
            to: "#00ff00".to_string(),
            angle: Some(45),
            relative_to: None,
            color_space: None,
        };
        assert!(gradient.to_string().contains("from=#ff0000"));
        assert!(gradient.to_string().contains("to=#00ff00"));
        assert!(gradient.to_string().contains("angle=45"));
    }

    #[test]
    fn test_center_focused_column_cycle() {
        let val = CenterFocusedColumn::Never;
        assert_eq!(val.next(), CenterFocusedColumn::Always);
        assert_eq!(val.next().next(), CenterFocusedColumn::OnOverflow);
        assert_eq!(val.next().next().next(), CenterFocusedColumn::Never);
    }

    #[test]
    fn test_view_model_visible_items() {
        let vm = AppearanceViewModel::new(AppearanceSettings::default());
        let items = vm.visible_items();

        // Should have section headers and their fields
        assert!(!items.is_empty());
        assert!(matches!(items[0], AppearanceListItem::SectionHeader(AppearanceSection::General)));
    }

    #[test]
    fn test_view_model_toggle_section() {
        let mut vm = AppearanceViewModel::new(AppearanceSettings::default());
        let initial_count = vm.visible_items().len();

        vm.toggle_section(AppearanceSection::General);
        let collapsed_count = vm.visible_items().len();

        // Should have fewer items when a section is collapsed
        assert!(collapsed_count < initial_count);

        vm.toggle_section(AppearanceSection::General);
        assert_eq!(vm.visible_items().len(), initial_count);
    }
}
