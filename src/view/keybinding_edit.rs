use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Widget},
};

use crate::model::{ActionType, EditField, EditMode};

/// Widget for editing a keybinding in a modal dialog
pub struct KeybindingEditWidget<'a> {
    edit_mode: &'a EditMode,
}

impl<'a> KeybindingEditWidget<'a> {
    pub fn new(edit_mode: &'a EditMode) -> Self {
        Self { edit_mode }
    }
}

impl Widget for KeybindingEditWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate centered dialog area
        let dialog_width = 65.min(area.width.saturating_sub(4));
        let dialog_height = 16.min(area.height.saturating_sub(2));
        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        Clear.render(dialog_area, buf);

        // Draw dialog border
        let title = if self.edit_mode.is_new {
            " Add Keybinding "
        } else {
            " Edit Keybinding "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        if inner.height < 10 || inner.width < 30 {
            return;
        }

        let label_style = Style::default().fg(Color::Gray);
        let value_style = Style::default().fg(Color::White);
        let focused_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let hint_style = Style::default().fg(Color::DarkGray);

        let mut y = inner.y;
        let input_width = (inner.width - 2) as usize;

        // Key Combo field
        let is_focused = self.edit_mode.focused_field == EditField::KeyCombo;
        buf.set_string(inner.x + 1, y, "Key Combo:", label_style);
        y += 1;

        let placeholder = if self.edit_mode.key_combo.is_empty() && is_focused {
            Some("e.g., Mod+Shift+T")
        } else {
            None
        };

        self.render_input_field(
            buf,
            inner.x + 1,
            y,
            input_width,
            &self.edit_mode.key_combo,
            self.edit_mode.key_combo_cursor,
            is_focused,
            placeholder,
        );
        y += 2;

        // Action Type selector
        let is_focused = self.edit_mode.focused_field == EditField::ActionType;
        buf.set_string(inner.x + 1, y, "Action Type:", label_style);
        y += 1;

        let type_display = format!(
            "< {} >",
            self.edit_mode.action_type.label()
        );
        let style = if is_focused { focused_style } else { value_style };
        buf.set_string(inner.x + 1, y, &type_display, style);
        if is_focused {
            buf.set_string(
                inner.x + 1 + type_display.len() as u16 + 2,
                y,
                "(←/→ to change)",
                hint_style,
            );
        }
        y += 2;

        // Action Value field
        let is_focused = self.edit_mode.focused_field == EditField::ActionValue;
        let value_label = match self.edit_mode.action_type {
            ActionType::Spawn => "Command:",
            ActionType::SpawnSh => "Shell Command:",
            ActionType::BuiltIn => "Action:",
        };
        buf.set_string(inner.x + 1, y, value_label, label_style);
        y += 1;

        let placeholder = if self.edit_mode.action_value.is_empty() && is_focused {
            Some(match self.edit_mode.action_type {
                ActionType::Spawn => "e.g., alacritty or firefox --new-window",
                ActionType::SpawnSh => "e.g., notify-send 'Hello'",
                ActionType::BuiltIn => "e.g., close-window or focus-workspace 1",
            })
        } else {
            None
        };

        self.render_input_field(
            buf,
            inner.x + 1,
            y,
            input_width,
            &self.edit_mode.action_value,
            self.edit_mode.action_value_cursor,
            is_focused,
            placeholder,
        );
        y += 2;

        // Properties section
        buf.set_string(inner.x + 1, y, "Properties:", label_style);
        y += 1;

        // Repeat toggle
        let is_focused = self.edit_mode.focused_field == EditField::Repeat;
        let repeat_value = match self.edit_mode.repeat {
            None => "[ ] repeat (default: enabled)",
            Some(true) => "[x] repeat",
            Some(false) => "[ ] repeat (disabled)",
        };
        let style = if is_focused { focused_style } else { value_style };
        buf.set_string(inner.x + 3, y, repeat_value, style);
        y += 1;

        // Allow when locked toggle
        let is_focused = self.edit_mode.focused_field == EditField::AllowWhenLocked;
        let locked_value = match self.edit_mode.allow_when_locked {
            None => "[ ] allow-when-locked (default: disabled)",
            Some(true) => "[x] allow-when-locked",
            Some(false) => "[ ] allow-when-locked (disabled)",
        };
        let style = if is_focused { focused_style } else { value_style };
        buf.set_string(inner.x + 3, y, locked_value, style);
        y += 2;

        // Help text
        if y < inner.y + inner.height {
            buf.set_string(
                inner.x + 1,
                y,
                "↑↓:Fields  ←→:Cursor  Enter:Save  Esc:Cancel",
                hint_style,
            );
        }
    }
}

impl KeybindingEditWidget<'_> {
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
        // Draw input box border indicators
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        buf.set_string(x, y, "[", border_style);
        buf.set_string(x + width as u16 + 1, y, "]", border_style);

        let inner_x = x + 1;
        let inner_width = width.saturating_sub(1);

        // Background style
        let bg_style = if focused {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default().bg(Color::Black)
        };

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
            // Show cursor at start if focused
            if focused {
                let cursor_style = Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black);
                buf.set_string(inner_x, y, " ", cursor_style);
            }
            return;
        }

        // Calculate visible portion of text based on cursor position
        let text_len = text.len();
        let visible_width = inner_width.saturating_sub(1); // Leave room for cursor at end

        // Calculate scroll offset to keep cursor visible
        let scroll_offset = if cursor_pos <= visible_width {
            0
        } else {
            cursor_pos - visible_width
        };

        // Get the visible portion of text
        let visible_end = (scroll_offset + visible_width).min(text_len);
        let visible_text = &text[scroll_offset..visible_end];

        // Text style
        let text_style = if focused {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };

        // Render text
        buf.set_string(inner_x, y, visible_text, text_style);

        // Show cursor if focused
        if focused {
            let cursor_screen_pos = cursor_pos - scroll_offset;
            let cursor_x = inner_x + cursor_screen_pos as u16;

            // Get character at cursor position (or space if at end)
            let cursor_char = if cursor_pos < text_len {
                text.chars().nth(cursor_pos).unwrap_or(' ')
            } else {
                ' '
            };

            let cursor_style = Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black);
            buf.set_string(cursor_x, y, &cursor_char.to_string(), cursor_style);
        }

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
