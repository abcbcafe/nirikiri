use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::model::OutputViewModel;

pub struct OutputListWidget<'a> {
    pub view_model: &'a OutputViewModel,
    pub focused: bool,
}

impl<'a> OutputListWidget<'a> {
    pub fn new(view_model: &'a OutputViewModel, focused: bool) -> Self {
        Self { view_model, focused }
    }
}

impl<'a> Widget for OutputListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .view_model
            .outputs
            .iter()
            .enumerate()
            .map(|(idx, output)| {
                let selected = idx == self.view_model.selected_index;
                let modified = self.view_model.pending_changes.contains_key(&output.name);

                let prefix = if selected { "> " } else { "  " };
                let suffix = if modified { " (*)" } else { "" };
                let enabled_indicator = if output.enabled { "" } else { " [off]" };

                let style = if !output.enabled {
                    Style::default().fg(Color::DarkGray)
                } else if selected && self.focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if selected {
                    Style::default().fg(Color::White)
                } else if modified {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(&output.name, style),
                    Span::styled(enabled_indicator, Style::default().fg(Color::DarkGray)),
                    Span::styled(suffix, Style::default().fg(Color::Cyan)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let list = List::new(items).block(
            Block::default()
                .title(" Outputs ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        let mut state = ListState::default();
        state.select(Some(self.view_model.selected_index));

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
