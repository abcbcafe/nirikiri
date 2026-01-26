use crate::message::Message;
use crate::model::KeybindingsViewModel;

/// Update keybindings view model based on message
#[allow(dead_code)] // Available for external use, currently handled in app.rs
pub fn update_keybindings(view_model: &mut KeybindingsViewModel, message: &Message) {
    match message {
        Message::SelectNextKeybinding => {
            view_model.select_next();
        }
        Message::SelectPrevKeybinding => {
            view_model.select_prev();
        }
        Message::SelectKeybinding(idx) => {
            let count = view_model.visible_count();
            if *idx < count {
                view_model.selected_index = *idx;
            }
        }
        Message::StartSearch => {
            view_model.search_mode = true;
        }
        Message::UpdateSearch(query) => {
            view_model.set_search(query.clone());
        }
        Message::ClearSearch => {
            view_model.clear_search();
        }
        _ => {}
    }
}
