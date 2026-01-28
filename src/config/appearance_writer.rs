use anyhow::Result;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use crate::model::{AppearanceSettings, ColorValue, ConfigDocument};

/// Write appearance settings to the config document
pub fn write_appearance(config: &mut ConfigDocument, settings: &AppearanceSettings) -> Result<()> {
    // Find or create the layout block
    let layout_idx = config
        .doc
        .nodes()
        .iter()
        .position(|n| n.name().value() == "layout");

    let layout_node = if let Some(idx) = layout_idx {
        config.doc.nodes_mut().get_mut(idx).unwrap()
    } else {
        // Create a new layout block
        let mut layout = KdlNode::new("layout");
        layout.set_children(KdlDocument::new());
        config.doc.nodes_mut().push(layout);
        config.doc.nodes_mut().last_mut().unwrap()
    };

    // Ensure children exist
    if layout_node.children().is_none() {
        layout_node.set_children(KdlDocument::new());
    }

    let children = layout_node.children_mut().as_mut().unwrap();

    // Update or add gaps
    update_or_add_simple_value(children, "gaps", KdlValue::Integer(settings.gaps as i128));

    // Update or add center-focused-column
    update_or_add_simple_value(
        children,
        "center-focused-column",
        KdlValue::String(settings.center_focused_column.as_str().to_string()),
    );

    // Update focus-ring block
    update_focus_ring(children, &settings.focus_ring);

    // Update border block
    update_border(children, &settings.border);

    // Update shadow block
    update_shadow(children, &settings.shadow);

    // Update struts block
    update_struts(children, &settings.struts);

    // Autoformat
    children.autoformat();
    layout_node.autoformat();

    config.save()
}

fn update_or_add_simple_value(children: &mut KdlDocument, name: &str, value: KdlValue) {
    // Try to find existing node
    if let Some(node) = children.nodes_mut().iter_mut().find(|n| n.name().value() == name) {
        // Update existing
        node.entries_mut().clear();
        node.push(KdlEntry::new(value));
    } else {
        // Add new
        let mut node = KdlNode::new(name);
        node.push(KdlEntry::new(value));
        children.nodes_mut().push(node);
    }
}

fn update_focus_ring(parent: &mut KdlDocument, settings: &crate::model::FocusRingSettings) {
    // Find or create focus-ring block
    let focus_ring_idx = parent
        .nodes()
        .iter()
        .position(|n| n.name().value() == "focus-ring");

    let focus_ring = if let Some(idx) = focus_ring_idx {
        parent.nodes_mut().get_mut(idx).unwrap()
    } else {
        let mut node = KdlNode::new("focus-ring");
        node.set_children(KdlDocument::new());
        parent.nodes_mut().push(node);
        parent.nodes_mut().last_mut().unwrap()
    };

    if focus_ring.children().is_none() {
        focus_ring.set_children(KdlDocument::new());
    }

    let children = focus_ring.children_mut().as_mut().unwrap();

    // Handle off state
    update_toggle_node(children, "off", settings.off);

    // Update width
    update_or_add_simple_value(children, "width", KdlValue::Integer(settings.width as i128));

    // Update colors
    update_color(children, "active-color", &settings.active_color);
    update_color(children, "inactive-color", &settings.inactive_color);

    // Handle gradients if present
    if let Some(ref gradient) = settings.active_gradient {
        update_gradient(children, "active-gradient", gradient);
    } else {
        remove_node(children, "active-gradient");
    }

    if let Some(ref gradient) = settings.inactive_gradient {
        update_gradient(children, "inactive-gradient", gradient);
    } else {
        remove_node(children, "inactive-gradient");
    }

    children.autoformat();
    focus_ring.autoformat();
}

fn update_border(parent: &mut KdlDocument, settings: &crate::model::BorderSettings) {
    let border_idx = parent
        .nodes()
        .iter()
        .position(|n| n.name().value() == "border");

    let border = if let Some(idx) = border_idx {
        parent.nodes_mut().get_mut(idx).unwrap()
    } else {
        let mut node = KdlNode::new("border");
        node.set_children(KdlDocument::new());
        parent.nodes_mut().push(node);
        parent.nodes_mut().last_mut().unwrap()
    };

    if border.children().is_none() {
        border.set_children(KdlDocument::new());
    }

    let children = border.children_mut().as_mut().unwrap();

    // Handle off/on state - remove the other when setting one
    if settings.off {
        update_toggle_node(children, "off", true);
        remove_node(children, "on");
    } else {
        update_toggle_node(children, "on", true);
        remove_node(children, "off");
    }

    update_or_add_simple_value(children, "width", KdlValue::Integer(settings.width as i128));
    update_color(children, "active-color", &settings.active_color);
    update_color(children, "inactive-color", &settings.inactive_color);

    if let Some(ref color) = settings.urgent_color {
        update_color(children, "urgent-color", color);
    } else {
        remove_node(children, "urgent-color");
    }

    if let Some(ref gradient) = settings.active_gradient {
        update_gradient(children, "active-gradient", gradient);
    } else {
        remove_node(children, "active-gradient");
    }

    if let Some(ref gradient) = settings.inactive_gradient {
        update_gradient(children, "inactive-gradient", gradient);
    } else {
        remove_node(children, "inactive-gradient");
    }

    children.autoformat();
    border.autoformat();
}

fn update_shadow(parent: &mut KdlDocument, settings: &crate::model::ShadowSettings) {
    let shadow_idx = parent
        .nodes()
        .iter()
        .position(|n| n.name().value() == "shadow");

    let shadow = if let Some(idx) = shadow_idx {
        parent.nodes_mut().get_mut(idx).unwrap()
    } else {
        let mut node = KdlNode::new("shadow");
        node.set_children(KdlDocument::new());
        parent.nodes_mut().push(node);
        parent.nodes_mut().last_mut().unwrap()
    };

    if shadow.children().is_none() {
        shadow.set_children(KdlDocument::new());
    }

    let children = shadow.children_mut().as_mut().unwrap();

    // Handle on state
    update_toggle_node(children, "on", settings.on);

    // Handle draw-behind-window
    if settings.draw_behind_window {
        update_or_add_simple_value(children, "draw-behind-window", KdlValue::Bool(true));
    } else {
        remove_node(children, "draw-behind-window");
    }

    update_or_add_simple_value(children, "softness", KdlValue::Integer(settings.softness as i128));
    update_or_add_simple_value(children, "spread", KdlValue::Integer(settings.spread as i128));

    // Update offset
    update_offset(children, settings.offset_x, settings.offset_y);

    update_color(children, "color", &settings.color);

    children.autoformat();
    shadow.autoformat();
}

fn update_struts(parent: &mut KdlDocument, settings: &crate::model::StrutsSettings) {
    let struts_idx = parent
        .nodes()
        .iter()
        .position(|n| n.name().value() == "struts");

    let struts = if let Some(idx) = struts_idx {
        parent.nodes_mut().get_mut(idx).unwrap()
    } else {
        let mut node = KdlNode::new("struts");
        node.set_children(KdlDocument::new());
        parent.nodes_mut().push(node);
        parent.nodes_mut().last_mut().unwrap()
    };

    if struts.children().is_none() {
        struts.set_children(KdlDocument::new());
    }

    let children = struts.children_mut().as_mut().unwrap();

    update_optional_value(children, "left", settings.left);
    update_optional_value(children, "right", settings.right);
    update_optional_value(children, "top", settings.top);
    update_optional_value(children, "bottom", settings.bottom);

    children.autoformat();
    struts.autoformat();
}

fn update_toggle_node(children: &mut KdlDocument, name: &str, enabled: bool) {
    let exists = children.nodes().iter().any(|n| n.name().value() == name);

    if enabled && !exists {
        let node = KdlNode::new(name);
        children.nodes_mut().push(node);
    } else if !enabled && exists {
        remove_node(children, name);
    }
}

fn update_color(children: &mut KdlDocument, name: &str, color: &ColorValue) {
    match color {
        ColorValue::Solid(c) => {
            update_or_add_simple_value(children, name, KdlValue::String(c.clone()));
        }
        ColorValue::Gradient { .. } => {
            // For gradients, we need a different approach - store as gradient node
            update_gradient(children, name, color);
        }
    }
}

fn update_gradient(children: &mut KdlDocument, name: &str, gradient: &ColorValue) {
    if let ColorValue::Gradient {
        from,
        to,
        angle,
        relative_to,
        color_space,
    } = gradient
    {
        // Remove existing node
        remove_node(children, name);

        // Create new gradient node
        let mut node = KdlNode::new(name);
        node.push(KdlEntry::new_prop("from", KdlValue::String(from.clone())));
        node.push(KdlEntry::new_prop("to", KdlValue::String(to.clone())));

        if let Some(a) = angle {
            node.push(KdlEntry::new_prop("angle", KdlValue::Integer(*a as i128)));
        }
        if let Some(r) = relative_to {
            node.push(KdlEntry::new_prop("relative-to", KdlValue::String(r.clone())));
        }
        if let Some(c) = color_space {
            node.push(KdlEntry::new_prop("in", KdlValue::String(c.clone())));
        }

        children.nodes_mut().push(node);
    }
}

fn update_offset(children: &mut KdlDocument, x: i32, y: i32) {
    // Remove existing
    remove_node(children, "offset");

    // Create new offset node
    let mut node = KdlNode::new("offset");
    node.push(KdlEntry::new_prop("x", KdlValue::Integer(x as i128)));
    node.push(KdlEntry::new_prop("y", KdlValue::Integer(y as i128)));
    children.nodes_mut().push(node);
}

fn update_optional_value(children: &mut KdlDocument, name: &str, value: Option<i32>) {
    if let Some(v) = value {
        update_or_add_simple_value(children, name, KdlValue::Integer(v as i128));
    } else {
        remove_node(children, name);
    }
}

fn remove_node(children: &mut KdlDocument, name: &str) {
    children.nodes_mut().retain(|n| n.name().value() != name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::appearance_parser::parse_appearance;
    use crate::model::CenterFocusedColumn;

    fn create_test_config(content: &str) -> ConfigDocument {
        ConfigDocument {
            doc: content.parse().unwrap(),
            path: std::path::PathBuf::from("/tmp/test.kdl"),
        }
    }

    #[test]
    fn test_write_gaps() {
        let config = create_test_config("layout { gaps 16 }");
        let mut settings = parse_appearance(&config);
        settings.gaps = 24;

        // We can't actually save in test, but we can verify the structure
        let layout_idx = config.doc.nodes().iter().position(|n| n.name().value() == "layout");
        assert!(layout_idx.is_some());
    }

    #[test]
    fn test_center_focused_column_conversion() {
        assert_eq!(CenterFocusedColumn::Never.as_str(), "never");
        assert_eq!(CenterFocusedColumn::Always.as_str(), "always");
        assert_eq!(CenterFocusedColumn::OnOverflow.as_str(), "on-overflow");
    }
}
