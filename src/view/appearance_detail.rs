use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::model::{AppearanceField, AppearanceListItem, AppearanceSection, AppearanceViewModel, ColorValue, FieldValue};

/// Parse a hex color string to a ratatui Color
fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');

    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color::Rgb(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color::Rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

/// Widget for displaying details of the selected appearance setting
pub struct AppearanceDetailWidget<'a> {
    view_model: &'a AppearanceViewModel,
}

impl<'a> AppearanceDetailWidget<'a> {
    pub fn new(view_model: &'a AppearanceViewModel) -> Self {
        Self { view_model }
    }
}

impl Widget for AppearanceDetailWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Details ");

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 || inner.width < 15 {
            return;
        }

        let Some(item) = self.view_model.selected_item() else {
            buf.set_string(
                inner.x + 1,
                inner.y + 1,
                "No setting selected",
                Style::default().fg(Color::DarkGray),
            );
            return;
        };

        match item {
            AppearanceListItem::SectionHeader(section) => {
                self.render_section_details(buf, inner, section);
            }
            AppearanceListItem::Field(field) => {
                self.render_field_details(buf, inner, field);
            }
        }
    }
}

impl AppearanceDetailWidget<'_> {
    fn render_section_details(&self, buf: &mut Buffer, area: Rect, section: AppearanceSection) {
        let label_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(Color::White);
        let dim_style = Style::default().fg(Color::DarkGray);

        let mut y = area.y;

        // Section name
        if y < area.y + area.height {
            buf.set_string(area.x + 1, y, "Section:", label_style);
            buf.set_string(area.x + 10, y, section.name(), value_style);
            y += 2;
        }

        // Description based on section
        let description = match section {
            AppearanceSection::General => "General layout settings including gaps and column centering behavior.",
            AppearanceSection::FocusRing => "Configure the visual ring around the focused window. The ring only shows on the active window.",
            AppearanceSection::Border => "Configure window borders that are always visible (unlike focus ring). Enable with 'on', disable with 'off'.",
            AppearanceSection::Shadow => "Configure drop shadows for windows. Enable with 'on'. Shadows are drawn behind windows.",
            AppearanceSection::Struts => "Configure outer gaps (struts) that shrink the usable window area, similar to panel margins.",
        };

        if y < area.y + area.height {
            buf.set_string(area.x + 1, y, "Description:", label_style);
            y += 1;
        }

        // Word-wrap the description
        let max_width = (area.width - 2) as usize;
        for line in wrap_text(description, max_width) {
            if y < area.y + area.height {
                buf.set_string(area.x + 1, y, &line, dim_style);
                y += 1;
            }
        }

        y += 1;

        // Show field count
        if y < area.y + area.height {
            let field_count = section.fields().len();
            buf.set_string(area.x + 1, y, "Settings:", label_style);
            buf.set_string(area.x + 11, y, &format!("{field_count}"), value_style);
            y += 1;
        }

        // Hint
        y += 1;
        if y < area.y + area.height {
            buf.set_string(
                area.x + 1,
                y,
                "Press Tab to expand/collapse",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            );
        }
    }

    fn render_field_details(&self, buf: &mut Buffer, area: Rect, field: AppearanceField) {
        let label_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(Color::White);
        let dim_style = Style::default().fg(Color::DarkGray);

        let mut y = area.y;

        // Field name
        if y < area.y + area.height {
            buf.set_string(area.x + 1, y, "Setting:", label_style);
            buf.set_string(area.x + 10, y, field.name(), value_style);
            y += 1;
        }

        // Section
        if y < area.y + area.height {
            buf.set_string(area.x + 1, y, "Section:", label_style);
            buf.set_string(area.x + 10, y, field.section().name(), dim_style);
            y += 2;
        }

        // Current value
        if y < area.y + area.height {
            let value = self.view_model.get_field_value(field);
            buf.set_string(area.x + 1, y, "Value:", label_style);

            let value_x = area.x + 8;
            match &value {
                FieldValue::Boolean(b) => {
                    // For "off" semantic fields, invert the display
                    let is_enabled = if field.is_off_semantic() { !*b } else { *b };

                    // Visual toggle display
                    let (toggle_text, toggle_fg, toggle_bg) = if is_enabled {
                        (" ON ", Color::Black, Color::Green)
                    } else {
                        ("OFF ", Color::White, Color::DarkGray)
                    };
                    let toggle_style = Style::default().fg(toggle_fg).bg(toggle_bg);
                    buf.set_string(value_x, y, toggle_text, toggle_style);
                }
                FieldValue::Color(color_value) => {
                    match color_value {
                        ColorValue::Solid(c) => {
                            // Show color preview
                            if let Some(color) = parse_hex_color(c) {
                                let preview_style = Style::default().bg(color);
                                buf.set_string(value_x, y, "    ", preview_style);
                                buf.set_string(value_x + 5, y, c, value_style);
                            } else {
                                buf.set_string(value_x, y, c, value_style);
                            }
                        }
                        ColorValue::Gradient { from, to, angle, .. } => {
                            // Show gradient info
                            buf.set_string(value_x, y, "gradient", value_style);
                            y += 1;
                            if y < area.y + area.height {
                                // Show from color with preview
                                buf.set_string(area.x + 3, y, "from:", dim_style);
                                if let Some(color) = parse_hex_color(from) {
                                    let preview_style = Style::default().bg(color);
                                    buf.set_string(area.x + 9, y, "  ", preview_style);
                                    buf.set_string(area.x + 12, y, from, value_style);
                                } else {
                                    buf.set_string(area.x + 9, y, from, value_style);
                                }
                            }
                            y += 1;
                            if y < area.y + area.height {
                                // Show to color with preview
                                buf.set_string(area.x + 3, y, "to:", dim_style);
                                if let Some(color) = parse_hex_color(to) {
                                    let preview_style = Style::default().bg(color);
                                    buf.set_string(area.x + 9, y, "  ", preview_style);
                                    buf.set_string(area.x + 12, y, to, value_style);
                                } else {
                                    buf.set_string(area.x + 9, y, to, value_style);
                                }
                            }
                            if let Some(a) = angle {
                                y += 1;
                                if y < area.y + area.height {
                                    buf.set_string(area.x + 3, y, &format!("angle: {a}°"), dim_style);
                                }
                            }
                        }
                    }
                }
                _ => {
                    let value_str = value.to_string();
                    let max_width = (area.width - 9) as usize;
                    let display = if value_str.len() > max_width {
                        format!("{}...", &value_str[..max_width.saturating_sub(3)])
                    } else {
                        value_str
                    };
                    buf.set_string(value_x, y, &display, value_style);
                }
            }
            y += 1;
        }

        // Large color preview for color fields
        if field.is_color() {
            let value = self.view_model.get_field_value(field);
            if let FieldValue::Color(ColorValue::Solid(ref c)) = value {
                if let Some(color) = parse_hex_color(c) {
                    y += 1;
                    if y + 2 < area.y + area.height {
                        buf.set_string(area.x + 1, y, "Preview:", label_style);
                        y += 1;
                        let preview_style = Style::default().bg(color);
                        let preview_width = (area.width - 4).min(20) as usize;
                        let preview_block = " ".repeat(preview_width);
                        // Draw 2 rows of preview
                        for _ in 0..2 {
                            if y < area.y + area.height {
                                buf.set_string(area.x + 2, y, &preview_block, preview_style);
                                y += 1;
                            }
                        }
                    }
                }
            }
        }

        // Show type
        if y < area.y + area.height {
            let type_str = if field.is_boolean() {
                "boolean"
            } else if field.is_enum() {
                "enum"
            } else if field.is_color() {
                "color"
            } else if field.is_integer() {
                "integer"
            } else {
                "string"
            };
            buf.set_string(area.x + 1, y, "Type:", label_style);
            buf.set_string(area.x + 7, y, type_str, dim_style);
            y += 2;
        }

        // Description
        if y < area.y + area.height {
            buf.set_string(area.x + 1, y, "Description:", label_style);
            y += 1;
        }

        let max_width = (area.width - 2) as usize;
        for line in wrap_text(field.description(), max_width) {
            if y < area.y + area.height {
                buf.set_string(area.x + 1, y, &line, dim_style);
                y += 1;
            }
        }

        y += 1;

        // Modification status
        if self.view_model.is_field_modified(field) {
            if y < area.y + area.height {
                buf.set_string(
                    area.x + 1,
                    y,
                    "* Modified (unsaved)",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                );
                y += 1;
            }
        }

        y += 1;

        // Edit hints based on type
        if y < area.y + area.height {
            let hint = if field.is_boolean() {
                "Space: Toggle on/off"
            } else if field.is_enum() {
                "Space/←/→: Cycle options"
            } else if field.is_integer() {
                "+/-: Adjust value, Enter: Edit"
            } else {
                "Enter: Edit value"
            };
            buf.set_string(
                area.x + 1,
                y,
                hint,
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            );
        }
    }
}

/// Simple word wrapping for text
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
