use crate::model::{
    BindingAction, BindingArg, BindingProperties, ConfigDocument, Keybinding, Modifiers,
};

/// Parse the binds section from the config
pub fn parse_keybindings(config: &ConfigDocument) -> Vec<Keybinding> {
    let mut bindings = Vec::new();

    // Find the binds block
    for node in config.doc.nodes() {
        if node.name().value() == "binds" {
            if let Some(children) = node.children() {
                for (idx, bind_node) in children.nodes().iter().enumerate() {
                    if let Some(binding) = parse_single_binding(bind_node, idx) {
                        bindings.push(binding);
                    }
                }
            }
            break;
        }
    }

    bindings
}

fn parse_single_binding(node: &kdl::KdlNode, index: usize) -> Option<Keybinding> {
    // Node name is the key combo (e.g., "Mod+T", "XF86AudioRaiseVolume")
    let combo = node.name().value();

    // Skip commented-out bindings
    if combo.starts_with("/-") {
        return None;
    }

    // Parse modifiers and key from combo
    let (modifiers, key) = Modifiers::parse(combo);

    // Parse properties from the node (repeat, cooldown-ms, allow-when-locked)
    let properties = parse_binding_properties(node);

    // Parse action from children
    let action = parse_binding_action(node)?;

    Some(Keybinding {
        modifiers,
        key,
        properties,
        action,
        kdl_index: Some(index),
    })
}

fn parse_binding_properties(node: &kdl::KdlNode) -> BindingProperties {
    let mut props = BindingProperties::default();

    for entry in node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "repeat" => {
                    if let Some(val) = entry.value().as_bool() {
                        props.repeat = Some(val);
                    }
                }
                "cooldown-ms" => {
                    if let Some(val) = entry.value().as_integer() {
                        props.cooldown_ms = Some(val as u32);
                    }
                }
                "allow-when-locked" => {
                    if let Some(val) = entry.value().as_bool() {
                        props.allow_when_locked = Some(val);
                    }
                }
                _ => {}
            }
        }
    }

    props
}

fn parse_binding_action(node: &kdl::KdlNode) -> Option<BindingAction> {
    let children = node.children()?;
    let action_node = children.nodes().first()?;
    let action_name = action_node.name().value();

    match action_name {
        "spawn" => {
            let args: Vec<String> = action_node
                .entries()
                .iter()
                .filter(|e| e.name().is_none()) // Only positional args
                .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
                .collect();
            if args.is_empty() {
                None
            } else {
                Some(BindingAction::Spawn(args))
            }
        }
        "spawn-sh" => {
            let cmd = action_node
                .get(0)
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())?;
            Some(BindingAction::SpawnSh(cmd))
        }
        _ => {
            // Check if there's an argument
            let entries: Vec<_> = action_node
                .entries()
                .iter()
                .filter(|e| e.name().is_none())
                .collect();

            if entries.is_empty() {
                Some(BindingAction::Simple(action_name.to_string()))
            } else {
                let arg = &entries[0];
                let binding_arg = if let Some(n) = arg.value().as_integer() {
                    BindingArg::Number(n as i64)
                } else if let Some(s) = arg.value().as_string() {
                    BindingArg::String(s.to_string())
                } else if let Some(b) = arg.value().as_bool() {
                    BindingArg::Bool(b)
                } else {
                    return Some(BindingAction::Simple(action_name.to_string()));
                };
                Some(BindingAction::WithArg(action_name.to_string(), binding_arg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifiers() {
        let (mods, key) = Modifiers::parse("Mod+Shift+T");
        assert!(mods.mod_key);
        assert!(mods.shift);
        assert!(!mods.ctrl);
        assert!(!mods.alt);
        assert_eq!(key, "T");

        let (mods, key) = Modifiers::parse("XF86AudioRaiseVolume");
        assert!(!mods.mod_key);
        assert!(!mods.shift);
        assert_eq!(key, "XF86AudioRaiseVolume");
    }
}
