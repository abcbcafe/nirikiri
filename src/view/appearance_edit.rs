use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Widget},
};

use crate::model::{AppearanceEditMode, AppearanceField, ColorEditField};

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

/// Widget for editing an appearance setting in a modal dialog
pub struct AppearanceEditWidget<'a> {
    edit_mode: &'a AppearanceEditMode,
}

impl<'a> AppearanceEditWidget<'a> {
    pub fn new(edit_mode: &'a AppearanceEditMode) -> Self {
        Self { edit_mode }
    }
}

impl Widget for AppearanceEditWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Check if this is a color edit
        if self.edit_mode.color_state.is_some() {
            self.render_color_editor(area, buf);
        } else {
            self.render_simple_editor(area, buf);
        }
    }
}

impl AppearanceEditWidget<'_> {
    fn render_simple_editor(&self, area: Rect, buf: &mut Buffer) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(2));
        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);
        Clear.render(dialog_area, buf);

        let title = format!(" Edit: {} ", self.edit_mode.field.name());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        if inner.height < 5 || inner.width < 20 {
            return;
        }

        let label_style = Style::default().fg(Color::Gray);
        let hint_style = Style::default().fg(Color::DarkGray);

        let mut y = inner.y;
        let input_width = (inner.width - 2) as usize;

        // Description
        if y < inner.y + inner.height {
            let desc = self.edit_mode.field.description();
            let max_width = inner.width.saturating_sub(2) as usize;
            let display = if desc.len() > max_width {
                format!("{}...", &desc[..max_width.saturating_sub(3)])
            } else {
                desc.to_string()
            };
            buf.set_string(inner.x + 1, y, &display, hint_style);
            y += 2;
        }

        // Input field label
        if y < inner.y + inner.height {
            let type_label = if self.edit_mode.field.is_integer() {
                "Value (integer):"
            } else {
                "Value:"
            };
            buf.set_string(inner.x + 1, y, type_label, label_style);
            y += 1;
        }

        // Input field
        if y < inner.y + inner.height {
            let placeholder = if self.edit_mode.value.is_empty() {
                Some(get_placeholder(self.edit_mode.field))
            } else {
                None
            };

            self.render_input_field(
                buf,
                inner.x + 1,
                y,
                input_width,
                &self.edit_mode.value,
                self.edit_mode.cursor,
                true,
                placeholder,
            );
            y += 2;
        }

        // Help text
        if y < inner.y + inner.height {
            buf.set_string(
                inner.x + 1,
                y,
                "Enter: Save  Esc: Cancel",
                hint_style,
            );
        }
    }

    fn render_color_editor(&self, area: Rect, buf: &mut Buffer) {
        let cs = self.edit_mode.color_state.as_ref().unwrap();

        // Larger dialog for color editing
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = if cs.is_gradient { 18 } else { 12 };
        let dialog_height = dialog_height.min(area.height.saturating_sub(2));
        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);
        Clear.render(dialog_area, buf);

        let title = format!(" Edit: {} ", self.edit_mode.field.name());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        if inner.height < 8 || inner.width < 30 {
            return;
        }

        let label_style = Style::default().fg(Color::Gray);
        let hint_style = Style::default().fg(Color::DarkGray);

        let mut y = inner.y;
        let input_width = (inner.width - 4) as usize;

        // Color type selector (Space toggles from any field)
        buf.set_string(inner.x + 1, y, "Type:", label_style);

        let solid_style = if !cs.is_gradient {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let gradient_style = if cs.is_gradient {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        buf.set_string(inner.x + 7, y, " Solid ", solid_style);
        buf.set_string(inner.x + 15, y, " Gradient ", gradient_style);
        y += 2;

        if cs.is_gradient {
            // Gradient editing
            self.render_gradient_fields(buf, inner, &mut y, input_width);
        } else {
            // Solid color editing
            self.render_solid_field(buf, inner, &mut y, input_width);
        }

        // Help text
        y += 1;
        if y < inner.y + inner.height {
            let help = if cs.is_gradient {
                "Tab/↑↓: Fields  Space: Toggle type  Enter: Save  Esc: Cancel"
            } else {
                "Tab: Switch field  Space: Toggle type  Enter: Save  Esc: Cancel"
            };
            buf.set_string(inner.x + 1, y, help, hint_style);
        }
    }

    fn render_solid_field(&self, buf: &mut Buffer, inner: Rect, y: &mut u16, input_width: usize) {
        let cs = self.edit_mode.color_state.as_ref().unwrap();
        let label_style = Style::default().fg(Color::Gray);
        let is_focused = cs.focused_field == ColorEditField::SolidColor;

        buf.set_string(inner.x + 1, *y, "Color:", label_style);
        *y += 1;

        // Color preview
        if let Some(color) = parse_hex_color(&cs.solid_color) {
            let preview_style = Style::default().bg(color);
            buf.set_string(inner.x + 1, *y, "    ", preview_style);
            buf.set_string(inner.x + 6, *y, " ", Style::default());
        }

        // Input field
        self.render_input_field(
            buf,
            inner.x + 7,
            *y,
            input_width - 6,
            &cs.solid_color,
            cs.solid_cursor,
            is_focused,
            Some("#rrggbb"),
        );
        *y += 2;

        // Large preview
        if let Some(color) = parse_hex_color(&cs.solid_color) {
            buf.set_string(inner.x + 1, *y, "Preview:", label_style);
            *y += 1;
            let preview_style = Style::default().bg(color);
            let preview_width = (inner.width - 4).min(20) as usize;
            let preview_block = " ".repeat(preview_width);
            for _ in 0..2 {
                if *y < inner.y + inner.height {
                    buf.set_string(inner.x + 2, *y, &preview_block, preview_style);
                    *y += 1;
                }
            }
        }
    }

    fn render_gradient_fields(&self, buf: &mut Buffer, inner: Rect, y: &mut u16, input_width: usize) {
        let cs = self.edit_mode.color_state.as_ref().unwrap();
        let label_style = Style::default().fg(Color::Gray);
        let focused_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let hint_style = Style::default().fg(Color::DarkGray);

        // From color
        let is_focused = cs.focused_field == ColorEditField::GradientFrom;
        let from_label_style = if is_focused { focused_style } else { label_style };
        buf.set_string(inner.x + 1, *y, "From:", from_label_style);

        if let Some(color) = parse_hex_color(&cs.gradient_from) {
            let preview_style = Style::default().bg(color);
            buf.set_string(inner.x + 7, *y, "  ", preview_style);
        }

        self.render_input_field(
            buf,
            inner.x + 10,
            *y,
            input_width - 9,
            &cs.gradient_from,
            cs.gradient_from_cursor,
            is_focused,
            Some("#rrggbb"),
        );
        *y += 2;

        // To color
        let is_focused = cs.focused_field == ColorEditField::GradientTo;
        let to_label_style = if is_focused { focused_style } else { label_style };
        buf.set_string(inner.x + 1, *y, "To:", to_label_style);

        if let Some(color) = parse_hex_color(&cs.gradient_to) {
            let preview_style = Style::default().bg(color);
            buf.set_string(inner.x + 7, *y, "  ", preview_style);
        }

        self.render_input_field(
            buf,
            inner.x + 10,
            *y,
            input_width - 9,
            &cs.gradient_to,
            cs.gradient_to_cursor,
            is_focused,
            Some("#rrggbb"),
        );
        *y += 2;

        // Angle
        let is_focused = cs.focused_field == ColorEditField::GradientAngle;
        let angle_label_style = if is_focused { focused_style } else { label_style };
        buf.set_string(inner.x + 1, *y, "Angle:", angle_label_style);

        self.render_input_field(
            buf,
            inner.x + 10,
            *y,
            8,
            &cs.gradient_angle,
            cs.gradient_angle_cursor,
            is_focused,
            Some("180"),
        );
        buf.set_string(inner.x + 20, *y, "degrees (0-360)", hint_style);
        *y += 2;

        // Relative to
        let is_focused = cs.focused_field == ColorEditField::GradientRelativeTo;
        let rel_label_style = if is_focused { focused_style } else { label_style };
        buf.set_string(inner.x + 1, *y, "Relative:", rel_label_style);

        let window_style = if cs.gradient_relative_to == "window" {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let workspace_style = if cs.gradient_relative_to == "workspace-view" {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        buf.set_string(inner.x + 11, *y, " window ", window_style);
        buf.set_string(inner.x + 20, *y, " workspace ", workspace_style);

        if is_focused {
            buf.set_string(inner.x + 32, *y, "(Space)", hint_style);
        }
        *y += 2;

        // Gradient preview
        buf.set_string(inner.x + 1, *y, "Preview:", label_style);
        *y += 1;

        // Draw a simple gradient preview (from left to right)
        if let (Some(from_color), Some(to_color)) =
            (parse_hex_color(&cs.gradient_from), parse_hex_color(&cs.gradient_to))
        {
            let preview_width = (inner.width - 4).min(24) as usize;
            if *y < inner.y + inner.height {
                for i in 0..preview_width {
                    let t = i as f32 / (preview_width - 1) as f32;
                    let blended = blend_colors(from_color, to_color, t);
                    let style = Style::default().bg(blended);
                    buf.set_string(inner.x + 2 + i as u16, *y, " ", style);
                }
                *y += 1;
            }
            if *y < inner.y + inner.height {
                for i in 0..preview_width {
                    let t = i as f32 / (preview_width - 1) as f32;
                    let blended = blend_colors(from_color, to_color, t);
                    let style = Style::default().bg(blended);
                    buf.set_string(inner.x + 2 + i as u16, *y, " ", style);
                }
                *y += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_input_field(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        width: usize,
        text: &str,
        cursor_pos: usize,
        focused: bool,
        placeholder: Option<&str>,
    ) {
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        buf.set_string(x, y, "[", border_style);
        buf.set_string(x + width as u16 + 1, y, "]", border_style);

        let inner_x = x + 1;
        let inner_width = width.saturating_sub(1);

        let bg_style = if focused {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default().bg(Color::Black)
        };

        let bg_fill = " ".repeat(inner_width);
        buf.set_string(inner_x, y, &bg_fill, bg_style);

        if text.is_empty() {
            if let Some(ph) = placeholder {
                let ph_display = if ph.len() > inner_width {
                    &ph[..inner_width]
                } else {
                    ph
                };
                let ph_style = bg_style.fg(Color::Gray);
                buf.set_string(inner_x, y, ph_display, ph_style);
            }
            if focused {
                let cursor_style = Style::default().bg(Color::Yellow).fg(Color::Black);
                buf.set_string(inner_x, y, " ", cursor_style);
            }
            return;
        }

        let text_len = text.len();
        let visible_width = inner_width.saturating_sub(1);

        let scroll_offset = if cursor_pos <= visible_width {
            0
        } else {
            cursor_pos - visible_width
        };

        let visible_end = (scroll_offset + visible_width).min(text_len);
        let visible_text = &text[scroll_offset..visible_end];

        let text_style = bg_style.fg(Color::White);
        buf.set_string(inner_x, y, visible_text, text_style);

        if focused {
            let cursor_screen_pos = cursor_pos - scroll_offset;
            let cursor_x = inner_x + cursor_screen_pos as u16;

            let cursor_char = if cursor_pos < text_len {
                text.chars().nth(cursor_pos).unwrap_or(' ')
            } else {
                ' '
            };

            let cursor_style = Style::default().bg(Color::Yellow).fg(Color::Black);
            buf.set_string(cursor_x, y, &cursor_char.to_string(), cursor_style);
        }

        if scroll_offset > 0 {
            let indicator_style = bg_style.fg(Color::Cyan);
            buf.set_string(inner_x, y, "«", indicator_style);
        }
        if visible_end < text_len {
            let indicator_style = bg_style.fg(Color::Cyan);
            buf.set_string(inner_x + inner_width as u16 - 1, y, "»", indicator_style);
        }
    }
}

fn get_placeholder(field: AppearanceField) -> &'static str {
    if field.is_integer() {
        "0"
    } else {
        ""
    }
}

/// Blend two RGB colors
fn blend_colors(from: Color, to: Color, t: f32) -> Color {
    if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (from, to) {
        let r = ((1.0 - t) * r1 as f32 + t * r2 as f32) as u8;
        let g = ((1.0 - t) * g1 as f32 + t * g2 as f32) as u8;
        let b = ((1.0 - t) * b1 as f32 + t * b2 as f32) as u8;
        Color::Rgb(r, g, b)
    } else {
        from
    }
}
