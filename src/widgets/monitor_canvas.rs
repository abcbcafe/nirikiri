use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::model::{OutputViewModel, Position, Size};

/// Viewport state for the canvas (zoom only, auto-fits to show all monitors)
#[derive(Debug, Clone)]
pub struct CanvasViewport {
    pub scale: f64,
}

impl Default for CanvasViewport {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl CanvasViewport {
    pub fn zoom_in(&mut self) {
        self.scale = (self.scale * 1.2).min(4.0);
    }

    pub fn zoom_out(&mut self) {
        self.scale = (self.scale / 1.2).max(0.25);
    }

    pub fn reset(&mut self) {
        self.scale = 1.0;
    }
}

pub struct MonitorCanvasWidget<'a> {
    pub view_model: &'a OutputViewModel,
    pub viewport: &'a CanvasViewport,
    pub focused: bool,
}

impl<'a> MonitorCanvasWidget<'a> {
    pub fn new(view_model: &'a OutputViewModel, viewport: &'a CanvasViewport, focused: bool) -> Self {
        Self {
            view_model,
            viewport,
            focused,
        }
    }

    /// Get the bounding box of all monitors (min_x, min_y, max_x, max_y)
    fn get_bounds(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for output in &self.view_model.outputs {
            if !output.enabled {
                continue;
            }
            let pos = self.view_model.get_display_position(&output.name).unwrap_or(output.position);
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x + output.logical_size.width as i32);
            max_y = max_y.max(pos.y + output.logical_size.height as i32);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Convert logical coordinates to screen coordinates
    /// Aligns top-left of bounding box to top-left of canvas
    fn to_screen(&self, pos: Position, canvas_area: Rect) -> (i32, i32) {
        let (min_x, min_y, _, _) = self.get_bounds();
        let scale = self.calculate_auto_scale(canvas_area) * self.viewport.scale;

        // Offset position by bounds minimum, then scale
        let rel_x = pos.x - min_x;
        let rel_y = pos.y - min_y;

        // Small padding from edge
        let padding = 1;
        let x = padding + (rel_x as f64 * scale) as i32;
        let y = padding + (rel_y as f64 * scale / 2.0) as i32; // /2 for aspect ratio

        (x, y)
    }

    fn calculate_auto_scale(&self, area: Rect) -> f64 {
        if self.view_model.outputs.is_empty() {
            return 0.05;
        }

        let (min_x, min_y, max_x, max_y) = self.get_bounds();

        let total_width = (max_x - min_x) as f64;
        let total_height = (max_y - min_y) as f64;

        if total_width <= 0.0 || total_height <= 0.0 {
            return 0.05;
        }

        // Leave padding for corner labels (2 rows top/bottom, 2 chars left/right)
        let available_width = (area.width as f64 - 4.0).max(1.0);
        let available_height = (area.height as f64 - 4.0).max(1.0) * 2.0; // *2 to compensate for char aspect ratio

        let scale_x = available_width / total_width;
        let scale_y = available_height / total_height;

        scale_x.min(scale_y).min(0.1) // Cap at reasonable scale
    }

    /// Draw a monitor rectangle
    #[allow(clippy::too_many_arguments)]
    fn draw_monitor(
        &self,
        buf: &mut Buffer,
        canvas_area: Rect,
        name: &str,
        pos: Position,
        size: Size,
        selected: bool,
        modified: bool,
    ) {
        let (screen_x, screen_y) = self.to_screen(pos, canvas_area);
        let scale = self.calculate_auto_scale(canvas_area) * self.viewport.scale;

        let width = ((size.width as f64 * scale) as u16).max(1);
        let height = ((size.height as f64 * scale / 2.0) as u16).max(1); // /2 for char aspect ratio

        // Determine colors
        let (border_color, fill_color, text_color) = if selected && self.focused {
            (Color::Yellow, Color::DarkGray, Color::Yellow)
        } else if selected {
            (Color::White, Color::DarkGray, Color::White)
        } else if modified {
            (Color::Cyan, Color::Black, Color::Cyan)
        } else {
            (Color::Gray, Color::Black, Color::White)
        };

        // Calculate actual screen positions
        let left = canvas_area.x as i32 + screen_x;
        let top = canvas_area.y as i32 + screen_y;

        // Draw the rectangle
        for dy in 0..height {
            for dx in 0..width {
                let x = left + dx as i32;
                let y = top + dy as i32;

                // Check bounds
                if x < canvas_area.x as i32
                    || x >= (canvas_area.x + canvas_area.width) as i32
                    || y < canvas_area.y as i32
                    || y >= (canvas_area.y + canvas_area.height) as i32
                {
                    continue;
                }

                let x = x as u16;
                let y = y as u16;

                let is_border = dy == 0 || dy == height - 1 || dx == 0 || dx == width - 1;

                if is_border {
                    let ch = if dy == 0 && dx == 0 {
                        '┌'
                    } else if dy == 0 && dx == width - 1 {
                        '┐'
                    } else if dy == height - 1 && dx == 0 {
                        '└'
                    } else if dy == height - 1 && dx == width - 1 {
                        '┘'
                    } else if dy == 0 || dy == height - 1 {
                        '─'
                    } else {
                        '│'
                    };
                    buf[(x, y)].set_char(ch).set_fg(border_color);
                } else {
                    buf[(x, y)].set_char(' ').set_bg(fill_color);
                }
            }
        }

        // Helper to draw text centered horizontally
        let draw_text = |buf: &mut Buffer, text: &str, y: i32, color: Color| {
            let text_x = left + ((width as i32 - text.len() as i32) / 2).max(1);
            for (i, ch) in text.chars().enumerate() {
                let x = text_x + i as i32;
                if x >= canvas_area.x as i32
                    && x < (canvas_area.x + canvas_area.width) as i32
                    && y >= canvas_area.y as i32
                    && y < (canvas_area.y + canvas_area.height) as i32
                    && x > left
                    && x < left + width as i32 - 1
                {
                    buf[(x as u16, y as u16)]
                        .set_char(ch)
                        .set_fg(color)
                        .set_bg(fill_color);
                }
            }
        };

        // Draw name centered vertically (or near top if tall enough)
        let name_y = if height >= 4 {
            top + 1
        } else {
            top + (height as i32 / 2)
        };
        draw_text(buf, name, name_y, text_color);

        // Draw position below name if there's room
        if height >= 3 {
            let pos_str = format!("{},{}", pos.x, pos.y);
            let pos_y = if height >= 4 { name_y + 1 } else { name_y };
            // Only draw position on separate line if room
            if height >= 4 {
                draw_text(buf, &pos_str, pos_y, Color::DarkGray);
            }
        }
    }
}

impl<'a> Widget for MonitorCanvasWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Get bounds for title
        let (min_x, min_y, max_x, max_y) = self.get_bounds();
        let title = if min_x != i32::MAX {
            format!(" Layout ({},{}) to ({},{}) ", min_x, min_y, max_x, max_y)
        } else {
            " Monitor Layout ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        // Clear inner area
        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_bg(Color::Black);
            }
        }

        // Draw each monitor
        for (idx, output) in self.view_model.outputs.iter().enumerate() {
            if !output.enabled {
                continue;
            }

            let pos = self.view_model.get_display_position(&output.name).unwrap_or(output.position);
            let selected = idx == self.view_model.selected_index;
            let modified = self.view_model.pending_changes.contains_key(&output.name);

            self.draw_monitor(
                buf,
                inner,
                &output.name,
                pos,
                output.logical_size,
                selected,
                modified,
            );
        }
    }
}
