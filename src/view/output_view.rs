use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::model::{OutputState, OutputViewModel, Position};

/// Info panel showing details about the selected output
pub struct OutputInfoWidget<'a> {
    pub output: Option<&'a OutputState>,
    pub pending_position: Option<Position>,
}

impl<'a> OutputInfoWidget<'a> {
    pub fn new(view_model: &'a OutputViewModel) -> Self {
        let output = view_model.selected_output();
        let pending_position = output.and_then(|o| view_model.pending_changes.get(&o.name).copied());
        Self {
            output,
            pending_position,
        }
    }
}

impl<'a> Widget for OutputInfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Output Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(output) = self.output {
            let pos = self.pending_position.unwrap_or(output.position);
            let modified = self.pending_position.is_some();

            let lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Gray)),
                    Span::styled(&output.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("Mode: ", Style::default().fg(Color::Gray)),
                    Span::styled(output.mode_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Scale: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.1}", output.scale), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Transform: ", Style::default().fg(Color::Gray)),
                    Span::styled(output.transform.as_str(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Position: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("X={}, Y={}", pos.x, pos.y),
                        if modified {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    if modified {
                        Span::styled(" (modified)", Style::default().fg(Color::Cyan))
                    } else {
                        Span::raw("")
                    },
                ]),
                Line::from(vec![
                    Span::styled("Logical Size: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{}x{}", output.logical_size.width, output.logical_size.height),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Make/Model: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{} {}", output.make, output.model),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ];

            let paragraph = Paragraph::new(lines);
            paragraph.render(inner, buf);
        } else {
            let no_output = Paragraph::new("No output selected")
                .style(Style::default().fg(Color::DarkGray));
            no_output.render(inner, buf);
        }
    }
}
