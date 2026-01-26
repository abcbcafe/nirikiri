use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct StatusBarWidget<'a> {
    pub has_changes: bool,
    pub error: Option<String>,
    pub keybinds: &'a [(&'static str, &'static str)],
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(
        has_changes: bool,
        error: Option<String>,
        keybinds: &'a [(&'static str, &'static str)],
    ) -> Self {
        Self {
            has_changes,
            error,
            keybinds,
        }
    }
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans: Vec<Span> = Vec::new();
        for (i, (key, action)) in self.keybinds.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("[{key}]"),
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
                Span::styled(
                    "Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(error.as_str(), Style::default().fg(Color::Red)),
            ]);
            if area.height > 1 {
                buf.set_line(area.x + 1, y + 1, &error_line, area.width.saturating_sub(2));
            }
        }
    }
}
