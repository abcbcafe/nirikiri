use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct StatusBarWidget {
    pub has_changes: bool,
    pub error: Option<String>,
}

impl StatusBarWidget {
    pub fn new(has_changes: bool, error: Option<String>) -> Self {
        Self { has_changes, error }
    }
}

impl Widget for StatusBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Key bindings help
        let keybinds = [
            ("[q]", "Quit"),
            ("[Tab]", "Select"),
            ("[hjkl]", "Move"),
            ("[HJKL]", "Snap"),
            ("[n]", "Normalize"),
            ("[s]", "Save"),
        ];

        let mut spans: Vec<Span> = Vec::new();
        for (i, (key, action)) in keybinds.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                *key,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(*action, Style::default().fg(Color::Gray)));
        }

        // Add status indicators
        if self.has_changes {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "[Modified]",
                Style::default().fg(Color::Cyan),
            ));
        }

        let help_line = Line::from(spans);
        let y = area.y;

        buf.set_line(area.x + 1, y, &help_line, area.width.saturating_sub(2));

        // Show error if present
        if let Some(error) = &self.error {
            let error_line = Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error.as_str(), Style::default().fg(Color::Red)),
            ]);
            if area.height > 1 {
                buf.set_line(area.x + 1, y + 1, &error_line, area.width.saturating_sub(2));
            }
        }
    }
}
