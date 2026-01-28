use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Widget},
};

use crate::model::{AppearanceEditMode, AppearanceField};

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
        // Calculate centered dialog area
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(2));
        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        Clear.render(dialog_area, buf);

        // Draw dialog border
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
            let type_label = if self.edit_mode.field.is_color() {
                "Color (e.g., #ff0000):"
            } else if self.edit_mode.field.is_integer() {
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
}

impl AppearanceEditWidget<'_> {
    fn render_input_field(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        width: usize,
        text: &str,
        cursor_pos: usize,
        placeholder: Option<&str>,
    ) {
        // Draw input box border indicators
        let border_style = Style::default().fg(Color::Yellow);
        buf.set_string(x, y, "[", border_style);
        buf.set_string(x + width as u16 + 1, y, "]", border_style);

        let inner_x = x + 1;
        let inner_width = width.saturating_sub(1);

        // Background style
        let bg_style = Style::default().bg(Color::DarkGray);

        // Fill background
        let bg_fill = " ".repeat(inner_width);
        buf.set_string(inner_x, y, &bg_fill, bg_style);

        // If empty and has placeholder, show it dimmed
        if text.is_empty() {
            if let Some(ph) = placeholder {
                let ph_display = if ph.len() > inner_width {
                    &ph[..inner_width]
                } else {
                    ph
                };
                let ph_style = Style::default().bg(Color::DarkGray).fg(Color::Gray);
                buf.set_string(inner_x, y, ph_display, ph_style);
            }
            // Show cursor at start
            let cursor_style = Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black);
            buf.set_string(inner_x, y, " ", cursor_style);
            return;
        }

        // Calculate visible portion of text based on cursor position
        let text_len = text.len();
        let visible_width = inner_width.saturating_sub(1);

        let scroll_offset = if cursor_pos <= visible_width {
            0
        } else {
            cursor_pos - visible_width
        };

        let visible_end = (scroll_offset + visible_width).min(text_len);
        let visible_text = &text[scroll_offset..visible_end];

        // Text style
        let text_style = Style::default().bg(Color::DarkGray).fg(Color::White);

        // Render text
        buf.set_string(inner_x, y, visible_text, text_style);

        // Show cursor
        let cursor_screen_pos = cursor_pos - scroll_offset;
        let cursor_x = inner_x + cursor_screen_pos as u16;

        let cursor_char = if cursor_pos < text_len {
            text.chars().nth(cursor_pos).unwrap_or(' ')
        } else {
            ' '
        };

        let cursor_style = Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black);
        buf.set_string(cursor_x, y, &cursor_char.to_string(), cursor_style);

        // Show scroll indicator if text is scrolled
        if scroll_offset > 0 {
            let indicator_style = Style::default().bg(Color::DarkGray).fg(Color::Cyan);
            buf.set_string(inner_x, y, "«", indicator_style);
        }
        if visible_end < text_len {
            let indicator_style = Style::default().bg(Color::DarkGray).fg(Color::Cyan);
            buf.set_string(inner_x + inner_width as u16 - 1, y, "»", indicator_style);
        }
    }
}

fn get_placeholder(field: AppearanceField) -> &'static str {
    if field.is_color() {
        "#rrggbb or #rrggbbaa"
    } else if field.is_integer() {
        "0"
    } else {
        ""
    }
}
