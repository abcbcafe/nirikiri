use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::model::Keybinding;

/// Widget for displaying details of a selected keybinding
pub struct KeybindingDetailWidget<'a> {
    binding: Option<&'a Keybinding>,
}

impl<'a> KeybindingDetailWidget<'a> {
    pub fn new(binding: Option<&'a Keybinding>) -> Self {
        Self { binding }
    }
}

impl Widget for KeybindingDetailWidget<'_> {
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

        let Some(binding) = self.binding else {
            buf.set_string(
                inner.x + 1,
                inner.y + 1,
                "No binding selected",
                Style::default().fg(Color::DarkGray),
            );
            return;
        };

        let label_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(Color::White);
        let dim_style = Style::default().fg(Color::DarkGray);

        let mut y = inner.y;

        // Key combo
        if y < inner.y + inner.height {
            buf.set_string(inner.x + 1, y, "Key Combo:", label_style);
            buf.set_string(inner.x + 12, y, binding.combo(), value_style);
            y += 1;
        }

        // Action
        if y < inner.y + inner.height {
            buf.set_string(inner.x + 1, y, "Action:", label_style);
            let action_str = binding.action.to_string();
            let max_width = (inner.width - 9) as usize;
            let display = if action_str.len() > max_width {
                format!("{}...", &action_str[..max_width.saturating_sub(3)])
            } else {
                action_str
            };
            buf.set_string(inner.x + 9, y, &display, value_style);
            y += 1;
        }

        // Properties header
        if y < inner.y + inner.height {
            y += 1; // blank line
            buf.set_string(inner.x + 1, y, "Properties:", label_style);
            y += 1;
        }

        // Repeat property
        if y < inner.y + inner.height {
            let repeat_val = match binding.properties.repeat {
                Some(true) => "true",
                Some(false) => "false",
                None => "true (default)",
            };
            let repeat_style = if binding.properties.repeat.is_some() {
                value_style
            } else {
                dim_style
            };
            buf.set_string(inner.x + 3, y, "repeat:", dim_style);
            buf.set_string(inner.x + 11, y, repeat_val, repeat_style);
            y += 1;
        }

        // Cooldown property
        if y < inner.y + inner.height {
            if let Some(ms) = binding.properties.cooldown_ms {
                buf.set_string(inner.x + 3, y, "cooldown:", dim_style);
                buf.set_string(inner.x + 13, y, format!("{ms}ms"), value_style);
                y += 1;
            }
        }

        // Allow when locked property
        if y < inner.y + inner.height {
            if let Some(allowed) = binding.properties.allow_when_locked {
                buf.set_string(inner.x + 3, y, "allow-when-locked:", dim_style);
                buf.set_string(
                    inner.x + 22,
                    y,
                    if allowed { "true" } else { "false" },
                    value_style,
                );
                y += 1;
            }
        }

        // Category
        if y + 1 < inner.y + inner.height {
            y += 1; // blank line
            buf.set_string(inner.x + 1, y, "Category:", label_style);
            buf.set_string(inner.x + 11, y, binding.action.category(), value_style);
        }
    }
}
