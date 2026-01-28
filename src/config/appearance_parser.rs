use crate::model::{
    AppearanceSettings, BorderSettings, CenterFocusedColumn, ColorValue, FocusRingSettings,
    ShadowSettings, StrutsSettings, ConfigDocument,
};

/// Parse appearance settings from the layout block in the config
pub fn parse_appearance(config: &ConfigDocument) -> AppearanceSettings {
    let mut settings = AppearanceSettings::default();

    // Find the layout block
    for node in config.doc.nodes() {
        if node.name().value() == "layout" {
            parse_layout_block(node, &mut settings);
            break;
        }
    }

    settings
}

fn parse_layout_block(node: &kdl::KdlNode, settings: &mut AppearanceSettings) {
    // Parse direct children of layout
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            match name {
                "gaps" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_integer()) {
                        settings.gaps = val as i32;
                    }
                }
                "center-focused-column" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_string()) {
                        if let Some(cfc) = CenterFocusedColumn::from_str(val) {
                            settings.center_focused_column = cfc;
                        }
                    }
                }
                "focus-ring" => {
                    settings.focus_ring = parse_focus_ring(child);
                }
                "border" => {
                    settings.border = parse_border(child);
                }
                "shadow" => {
                    settings.shadow = parse_shadow(child);
                }
                "struts" => {
                    settings.struts = parse_struts(child);
                }
                _ => {}
            }
        }
    }
}

fn parse_focus_ring(node: &kdl::KdlNode) -> FocusRingSettings {
    let mut settings = FocusRingSettings::default();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            match name {
                "off" => {
                    settings.off = true;
                }
                "width" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_integer()) {
                        settings.width = val as i32;
                    }
                }
                "active-color" => {
                    if let Some(color) = parse_color_value(child) {
                        settings.active_color = color;
                    }
                }
                "inactive-color" => {
                    if let Some(color) = parse_color_value(child) {
                        settings.inactive_color = color;
                    }
                }
                "active-gradient" => {
                    // Gradient takes precedence over solid color - store in main color field
                    if let Some(gradient) = parse_gradient(child) {
                        settings.active_color = gradient.clone();
                        settings.active_gradient = Some(gradient);
                    }
                }
                "inactive-gradient" => {
                    // Gradient takes precedence over solid color - store in main color field
                    if let Some(gradient) = parse_gradient(child) {
                        settings.inactive_color = gradient.clone();
                        settings.inactive_gradient = Some(gradient);
                    }
                }
                _ => {}
            }
        }
    }

    settings
}

fn parse_border(node: &kdl::KdlNode) -> BorderSettings {
    let mut settings = BorderSettings::default();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            match name {
                "off" => {
                    settings.off = true;
                }
                "on" => {
                    settings.off = false;
                }
                "width" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_integer()) {
                        settings.width = val as i32;
                    }
                }
                "active-color" => {
                    if let Some(color) = parse_color_value(child) {
                        settings.active_color = color;
                    }
                }
                "inactive-color" => {
                    if let Some(color) = parse_color_value(child) {
                        settings.inactive_color = color;
                    }
                }
                "urgent-color" => {
                    settings.urgent_color = parse_color_value(child);
                }
                "active-gradient" => {
                    // Gradient takes precedence over solid color - store in main color field
                    if let Some(gradient) = parse_gradient(child) {
                        settings.active_color = gradient.clone();
                        settings.active_gradient = Some(gradient);
                    }
                }
                "inactive-gradient" => {
                    // Gradient takes precedence over solid color - store in main color field
                    if let Some(gradient) = parse_gradient(child) {
                        settings.inactive_color = gradient.clone();
                        settings.inactive_gradient = Some(gradient);
                    }
                }
                "urgent-gradient" => {
                    // Gradient takes precedence over solid color - store in main color field
                    if let Some(gradient) = parse_gradient(child) {
                        settings.urgent_color = Some(gradient);
                    }
                }
                _ => {}
            }
        }
    }

    settings
}

fn parse_shadow(node: &kdl::KdlNode) -> ShadowSettings {
    let mut settings = ShadowSettings::default();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            match name {
                "on" => {
                    settings.on = true;
                }
                "draw-behind-window" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_bool()) {
                        settings.draw_behind_window = val;
                    } else {
                        // If present without value, it means true
                        settings.draw_behind_window = true;
                    }
                }
                "softness" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_integer()) {
                        settings.softness = val as i32;
                    }
                }
                "spread" => {
                    if let Some(val) = child.get(0).and_then(|v| v.as_integer()) {
                        settings.spread = val as i32;
                    }
                }
                "offset" => {
                    // offset x=0 y=5
                    if let Some(x) = child.get("x").and_then(|v| v.as_integer()) {
                        settings.offset_x = x as i32;
                    }
                    if let Some(y) = child.get("y").and_then(|v| v.as_integer()) {
                        settings.offset_y = y as i32;
                    }
                }
                "color" => {
                    if let Some(color) = parse_color_value(child) {
                        settings.color = color;
                    }
                }
                _ => {}
            }
        }
    }

    settings
}

fn parse_struts(node: &kdl::KdlNode) -> StrutsSettings {
    let mut settings = StrutsSettings::default();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            let value = child.get(0).and_then(|v| v.as_integer()).map(|v| v as i32);

            match name {
                "left" => settings.left = value,
                "right" => settings.right = value,
                "top" => settings.top = value,
                "bottom" => settings.bottom = value,
                _ => {}
            }
        }
    }

    settings
}

/// Parse a color value from a node (either solid color string or gradient)
fn parse_color_value(node: &kdl::KdlNode) -> Option<ColorValue> {
    // First positional argument is the color string
    if let Some(color) = node.get(0).and_then(|v| v.as_string()) {
        return Some(ColorValue::Solid(color.to_string()));
    }
    None
}

/// Parse a gradient from named parameters
fn parse_gradient(node: &kdl::KdlNode) -> Option<ColorValue> {
    let from = node.get("from").and_then(|v| v.as_string())?.to_string();
    let to = node.get("to").and_then(|v| v.as_string())?.to_string();
    let angle = node.get("angle").and_then(|v| v.as_integer()).map(|v| v as i32);
    let relative_to = node.get("relative-to").and_then(|v| v.as_string()).map(|s| s.to_string());
    let color_space = node.get("in").and_then(|v| v.as_string()).map(|s| s.to_string());

    Some(ColorValue::Gradient {
        from,
        to,
        angle,
        relative_to,
        color_space,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ConfigDocument;

    fn parse_test_config(content: &str) -> ConfigDocument {
        ConfigDocument {
            doc: content.parse().unwrap(),
            path: std::path::PathBuf::new(),
        }
    }

    #[test]
    fn test_parse_gaps() {
        let config = parse_test_config(r#"
            layout {
                gaps 24
            }
        "#);
        let settings = parse_appearance(&config);
        assert_eq!(settings.gaps, 24);
    }

    #[test]
    fn test_parse_center_focused_column() {
        let config = parse_test_config(r#"
            layout {
                center-focused-column "on-overflow"
            }
        "#);
        let settings = parse_appearance(&config);
        assert_eq!(settings.center_focused_column, CenterFocusedColumn::OnOverflow);
    }

    #[test]
    fn test_parse_focus_ring() {
        let config = parse_test_config(r##"
            layout {
                focus-ring {
                    width 6
                    active-color "#ff0000"
                    inactive-color "#00ff00"
                }
            }
        "##);
        let settings = parse_appearance(&config);
        assert_eq!(settings.focus_ring.width, 6);
        assert_eq!(settings.focus_ring.active_color, ColorValue::Solid("#ff0000".to_string()));
        assert_eq!(settings.focus_ring.inactive_color, ColorValue::Solid("#00ff00".to_string()));
    }

    #[test]
    fn test_parse_shadow() {
        let config = parse_test_config(r##"
            layout {
                shadow {
                    on
                    softness 40
                    spread 10
                    offset x=5 y=10
                    color "#0005"
                }
            }
        "##);
        let settings = parse_appearance(&config);
        assert!(settings.shadow.on);
        assert_eq!(settings.shadow.softness, 40);
        assert_eq!(settings.shadow.spread, 10);
        assert_eq!(settings.shadow.offset_x, 5);
        assert_eq!(settings.shadow.offset_y, 10);
        assert_eq!(settings.shadow.color, ColorValue::Solid("#0005".to_string()));
    }

    #[test]
    fn test_parse_struts() {
        let config = parse_test_config(r#"
            layout {
                struts {
                    left 64
                    right 64
                }
            }
        "#);
        let settings = parse_appearance(&config);
        assert_eq!(settings.struts.left, Some(64));
        assert_eq!(settings.struts.right, Some(64));
        assert_eq!(settings.struts.top, None);
        assert_eq!(settings.struts.bottom, None);
    }

    #[test]
    fn test_parse_border_gradient() {
        let config = parse_test_config(r##"
            layout {
                border {
                    on
                    width 4
                    active-gradient from="#ff0000" to="#00ff00" angle=45
                    inactive-color "#505050"
                }
            }
        "##);
        let settings = parse_appearance(&config);
        assert!(!settings.border.off);
        assert_eq!(settings.border.width, 4);
        // Gradient should be stored in active_color field
        match &settings.border.active_color {
            ColorValue::Gradient { from, to, angle, .. } => {
                assert_eq!(from, "#ff0000");
                assert_eq!(to, "#00ff00");
                assert_eq!(*angle, Some(45));
            }
            _ => panic!("Expected gradient in active_color"),
        }
        assert_eq!(settings.border.inactive_color, ColorValue::Solid("#505050".to_string()));
    }
}
