use crate::message::Message;
use crate::model::{OutputViewModel, Position, Size};

/// Get the reference monitor (first other enabled monitor) for snap operations
fn get_reference_monitor(view_model: &OutputViewModel) -> Option<(Position, Size)> {
    let selected = view_model.selected_output()?;
    let selected_name = &selected.name;

    // Find first other enabled monitor as reference
    for output in &view_model.outputs {
        if &output.name == selected_name || !output.enabled {
            continue;
        }
        let pos = view_model.get_display_position(&output.name).unwrap_or(output.position);
        return Some((pos, output.logical_size));
    }
    None
}

/// Process output-related messages
pub fn update_output(view_model: &mut OutputViewModel, message: &Message) -> Option<Message> {
    match message {
        Message::SelectNextOutput => {
            view_model.select_next();
            None
        }
        Message::SelectPrevOutput => {
            view_model.select_prev();
            None
        }
        Message::SelectOutput(idx) => {
            if *idx < view_model.outputs.len() {
                view_model.selected_index = *idx;
            }
            None
        }
        Message::MoveOutput { dx, dy } => {
            if let Some(output) = view_model.selected_output() {
                let name = output.name.clone();
                let current_pos = view_model
                    .pending_changes
                    .get(&name)
                    .copied()
                    .unwrap_or(output.position);

                let new_pos = Position::new(current_pos.x + dx, current_pos.y + dy);
                view_model.apply_pending_change(&name, new_pos);
            }
            None
        }
        Message::SetPosition { x, y } => {
            if let Some(output) = view_model.selected_output() {
                let name = output.name.clone();
                view_model.apply_pending_change(&name, Position::new(*x, *y));
            }
            None
        }
        Message::SnapLeft => {
            if let (Some(output), Some((ref_pos, _ref_size))) =
                (view_model.selected_output(), get_reference_monitor(view_model))
            {
                let name = output.name.clone();
                let my_size = output.logical_size;
                // Place to the left of reference, align top edges
                let new_x = ref_pos.x - my_size.width as i32;
                let new_y = ref_pos.y;
                view_model.apply_pending_change(&name, Position::new(new_x, new_y));
            }
            None
        }
        Message::SnapRight => {
            if let (Some(output), Some((ref_pos, ref_size))) =
                (view_model.selected_output(), get_reference_monitor(view_model))
            {
                let name = output.name.clone();
                // Place to the right of reference, align top edges
                let new_x = ref_pos.x + ref_size.width as i32;
                let new_y = ref_pos.y;
                view_model.apply_pending_change(&name, Position::new(new_x, new_y));
            }
            None
        }
        Message::SnapAbove => {
            if let (Some(output), Some((ref_pos, ref_size))) =
                (view_model.selected_output(), get_reference_monitor(view_model))
            {
                let name = output.name.clone();
                let my_size = output.logical_size;
                // Center horizontally relative to reference, place above
                let new_x = ref_pos.x + (ref_size.width as i32 - my_size.width as i32) / 2;
                let new_y = ref_pos.y - my_size.height as i32;
                view_model.apply_pending_change(&name, Position::new(new_x, new_y));
            }
            None
        }
        Message::SnapBelow => {
            if let (Some(output), Some((ref_pos, ref_size))) =
                (view_model.selected_output(), get_reference_monitor(view_model))
            {
                let name = output.name.clone();
                let my_size = output.logical_size;
                // Center horizontally relative to reference, place below
                let new_x = ref_pos.x + (ref_size.width as i32 - my_size.width as i32) / 2;
                let new_y = ref_pos.y + ref_size.height as i32;
                view_model.apply_pending_change(&name, Position::new(new_x, new_y));
            }
            None
        }
        Message::Normalize => {
            // Find minimum x and y across all enabled outputs
            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;

            for output in &view_model.outputs {
                if !output.enabled {
                    continue;
                }
                let pos = view_model.get_display_position(&output.name).unwrap_or(output.position);
                min_x = min_x.min(pos.x);
                min_y = min_y.min(pos.y);
            }

            if min_x == i32::MAX || min_y == i32::MAX {
                return None;
            }

            // Collect changes first to avoid borrow issues
            let changes: Vec<_> = view_model
                .outputs
                .iter()
                .filter(|o| o.enabled)
                .map(|output| {
                    let current = view_model.get_display_position(&output.name).unwrap_or(output.position);
                    (output.name.clone(), Position::new(current.x - min_x, current.y - min_y))
                })
                .collect();

            // Apply changes
            for (name, new_pos) in changes {
                view_model.apply_pending_change(&name, new_pos);
            }
            None
        }
        _ => None,
    }
}
