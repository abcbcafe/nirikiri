use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::model::KeybindingsViewModel;

/// Widget for displaying the list of keybindings
pub struct KeybindingsListWidget<'a> {
    view_model: &'a KeybindingsViewModel,
    focused: bool,
}

impl<'a> KeybindingsListWidget<'a> {
    pub fn new(view_model: &'a KeybindingsViewModel, focused: bool) -> Self {
        Self { view_model, focused }
    }
}

impl Widget for KeybindingsListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filtered = self.view_model.filtered_bindings();
        let count = filtered.len();

        // Draw border with count
        let title = if self.view_model.search_query.is_empty() {
            format!(" Keybindings ({count}) ")
        } else {
            format!(" Keybindings ({}) [/{}] ", count, self.view_model.search_query)
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
        for (i, (_, binding)) in filtered
            .iter()
            .skip(scroll_offset)
            .take(visible_height)
            .enumerate()
        {
            let y = inner.y + i as u16;
            let is_selected = scroll_offset + i == self.view_model.selected_index;

            // Selection indicator
            let indicator = if is_selected { "> " } else { "  " };

            // Key combo (left-aligned, max width)
            let combo = binding.combo();
            let combo_width = 18.min(inner.width as usize - 3);
            let combo_display = if combo.len() > combo_width {
                format!("{}...", &combo[..combo_width - 3])
            } else {
                format!("{combo:combo_width$}")
            };

            // Action description (right side)
            let action_desc = binding.action.short_description();
            let action_width = inner.width as usize - combo_width - 4;
            let action_display = if action_desc.len() > action_width {
                format!("{}...", &action_desc[..action_width.saturating_sub(3)])
            } else {
                action_desc
            };

            // Style based on selection
            let style = if is_selected && self.focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            let action_style = if is_selected && self.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Render the line
            buf.set_string(inner.x, y, indicator, style);
            buf.set_string(inner.x + 2, y, &combo_display, style);
            buf.set_string(
                inner.x + 2 + combo_width as u16 + 1,
                y,
                &action_display,
                action_style,
            );
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
