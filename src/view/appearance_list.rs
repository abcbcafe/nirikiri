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
        // #RGB
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color::Rgb(r, g, b))
        }
        // #RGBA
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color::Rgb(r, g, b))
        }
        // #RRGGBB
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        // #RRGGBBAA
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

/// Widget for displaying the list of appearance settings with collapsible sections
pub struct AppearanceListWidget<'a> {
    view_model: &'a AppearanceViewModel,
    focused: bool,
}

impl<'a> AppearanceListWidget<'a> {
    pub fn new(view_model: &'a AppearanceViewModel, focused: bool) -> Self {
        Self { view_model, focused }
    }
}

impl Widget for AppearanceListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.view_model.visible_items();
        let count = items.len();

        // Draw border with count
        let modified_count = self.view_model.pending_changes.len();
        let title = if modified_count > 0 {
            format!(" Appearance ({count}) *{modified_count} modified ")
        } else {
            format!(" Appearance ({count}) ")
        };

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 1 || inner.width < 10 {
            return;
        }

        // Calculate visible range
        let visible_height = inner.height as usize;
        let scroll_offset = self.view_model.scroll_offset;

        // Render visible items
        for (i, item) in items
            .iter()
            .skip(scroll_offset)
            .take(visible_height)
            .enumerate()
        {
            let y = inner.y + i as u16;
            let is_selected = scroll_offset + i == self.view_model.selected_index;

            match item {
                AppearanceListItem::SectionHeader(section) => {
                    self.render_section_header(buf, inner.x, y, inner.width, *section, is_selected);
                }
                AppearanceListItem::Field(field) => {
                    let is_modified = self.view_model.is_field_modified(*field);
                    let value = self.view_model.get_field_value(*field);
                    self.render_field(
                        buf,
                        inner.x,
                        y,
                        inner.width,
                        *field,
                        &value,
                        is_selected,
                        is_modified,
                    );
                }
            }
        }

        // Show scroll indicators if needed
        if scroll_offset > 0 {
            buf.set_string(
                inner.x + inner.width - 3,
                inner.y,
                "▲",
                Style::default().fg(Color::DarkGray),
            );
        }
        if scroll_offset + visible_height < count {
            buf.set_string(
                inner.x + inner.width - 3,
                inner.y + inner.height - 1,
                "▼",
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

impl AppearanceListWidget<'_> {
    fn render_section_header(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        width: u16,
        section: AppearanceSection,
        is_selected: bool,
    ) {
        let is_collapsed = self.view_model.collapsed_sections.contains(&section);
        let collapse_char = if is_collapsed { "▶" } else { "▼" };
        let name = section.name();

        // Selection indicator
        let indicator = if is_selected { ">" } else { " " };

        let style = if is_selected && self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };

        // Clear the line
        let clear = " ".repeat(width as usize);
        buf.set_string(x, y, &clear, Style::default());

        // Render: "> ▶ Section Name" or "> ▼ Section Name"
        buf.set_string(x, y, indicator, style);
        buf.set_string(x + 2, y, collapse_char, style);
        buf.set_string(x + 4, y, name, style);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_field(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        width: u16,
        field: AppearanceField,
        value: &FieldValue,
        is_selected: bool,
        is_modified: bool,
    ) {
        let name = field.name();

        // Selection and modification indicators
        let indicator = match (is_selected, is_modified) {
            (true, true) => ">*",
            (true, false) => "> ",
            (false, true) => " *",
            (false, false) => "  ",
        };

        // Calculate widths - reserve space for color preview if needed
        let has_color_preview = field.is_color();
        let color_preview_width = if has_color_preview { 4 } else { 0 }; // "██ "
        let available_width = width.saturating_sub(4 + color_preview_width as u16) as usize;
        let name_width = (available_width * 55 / 100).min(name.len() + 2);
        let value_width = available_width.saturating_sub(name_width);

        // Truncate name if needed
        let name_display = if name.len() > name_width {
            format!("{}...", &name[..name_width.saturating_sub(3)])
        } else {
            format!("{name:name_width$}")
        };

        // Styles
        let name_style = if is_selected && self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::White)
        } else if is_modified {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if is_selected && self.focused {
            Style::default().fg(Color::Yellow)
        } else if is_modified {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let indicator_style = if is_modified {
            Style::default().fg(Color::Cyan)
        } else {
            name_style
        };

        // Clear the line
        let clear = " ".repeat(width as usize);
        buf.set_string(x, y, &clear, Style::default());

        // Render indicator and name
        buf.set_string(x + 2, y, indicator, indicator_style);
        buf.set_string(x + 4, y, &name_display, name_style);

        let value_x = x + 4 + name_width as u16;

        // Render value based on type
        match value {
            FieldValue::Boolean(b) => {
                // For "off" semantic fields, invert the display
                // (e.g., FocusRingOff=true means focus ring is OFF)
                let is_enabled = if field.is_off_semantic() { !*b } else { *b };

                // Visual toggle: [ON ] or [OFF]
                let (toggle_text, toggle_fg, toggle_bg) = if is_enabled {
                    (" ON ", Color::Black, Color::Green)
                } else {
                    ("OFF ", Color::White, Color::DarkGray)
                };
                let toggle_style = Style::default().fg(toggle_fg).bg(toggle_bg);
                buf.set_string(value_x, y, toggle_text, toggle_style);
            }
            FieldValue::Color(color_value) => {
                // Get the color string
                let color_str = match color_value {
                    ColorValue::Solid(c) => c.clone(),
                    ColorValue::Gradient { from, .. } => from.clone(),
                };

                // Render color preview block
                if let Some(color) = parse_hex_color(&color_str) {
                    let preview_style = Style::default().bg(color);
                    buf.set_string(value_x, y, "  ", preview_style);
                }

                // Render color value text
                let text_x = value_x + 3;
                let remaining_width = value_width.saturating_sub(3);
                let value_display = if color_str.len() > remaining_width {
                    format!("{}...", &color_str[..remaining_width.saturating_sub(3)])
                } else {
                    color_str
                };
                buf.set_string(text_x, y, &value_display, value_style);
            }
            FieldValue::Enum(e) => {
                // Show enum with arrows to indicate it's cyclable
                let enum_display = format!("◀ {} ▶", e);
                let display = if enum_display.len() > value_width {
                    e.to_string()
                } else {
                    enum_display
                };
                buf.set_string(value_x, y, &display, value_style);
            }
            _ => {
                // Default: just show the value string
                let value_str = value.to_string();
                let value_display = if value_str.len() > value_width {
                    format!("{}...", &value_str[..value_width.saturating_sub(3)])
                } else {
                    value_str
                };
                buf.set_string(value_x, y, &value_display, value_style);
            }
        }
    }
}
