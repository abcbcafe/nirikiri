use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::category::Category;

/// Tab bar showing available settings categories with function key shortcuts
pub struct TabBarWidget {
    current: Category,
}

impl TabBarWidget {
    pub fn new(current: Category) -> Self {
        Self { current }
    }
}

impl Widget for TabBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 1 {
            return;
        }

        let mut x = area.x + 1;

        for category in Category::all() {
            let is_selected = *category == self.current;
            let fkey = category.function_key();
            let name = category.name();

            // Format: [F1] Outputs
            let tab_text = format!("[F{fkey}] {name}");
            let tab_width = tab_text.len() as u16;

            if x + tab_width > area.x + area.width - 1 {
                break;
            }

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            buf.set_string(x, area.y, &tab_text, style);
            x += tab_width + 2; // Add spacing between tabs

            // Add separator unless it's the last tab
            if x < area.x + area.width - 1 {
                buf.set_string(x - 2, area.y, "|", Style::default().fg(Color::DarkGray));
            }
        }

        // Fill rest with border
        let border_style = Style::default().fg(Color::DarkGray);
        for x_pos in x..area.x + area.width {
            buf.set_string(x_pos, area.y, "─", border_style);
        }
    }
}
