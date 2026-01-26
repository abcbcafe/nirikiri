use anyhow::{Context, Result};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use crate::model::{
    BindingAction, BindingArg, ConfigDocument, Keybinding, KeybindingChange,
};

/// Apply keybinding changes to the config document
pub fn write_keybindings(
    config: &mut ConfigDocument,
    changes: &[KeybindingChange],
) -> Result<()> {
    // Find the binds block
    let binds_idx = config
        .doc
        .nodes()
        .iter()
        .position(|n| n.name().value() == "binds")
        .context("No binds block found in config")?;

    let binds_node = config.doc.nodes_mut().get_mut(binds_idx).unwrap();

    // Ensure children exist
    if binds_node.children().is_none() {
        binds_node.set_children(KdlDocument::new());
    }

    let children = binds_node.children_mut().as_mut().unwrap();

    // Process changes in reverse order (deletes first, then modifies, then adds)
    // This preserves indices during deletion
    let mut sorted_changes: Vec<_> = changes.iter().collect();
    sorted_changes.sort_by(|a, b| {
        match (a, b) {
            (KeybindingChange::Delete(i1), KeybindingChange::Delete(i2)) => i2.cmp(i1), // Delete in reverse order
            (KeybindingChange::Delete(_), _) => std::cmp::Ordering::Less,
            (_, KeybindingChange::Delete(_)) => std::cmp::Ordering::Greater,
            (KeybindingChange::Modify { index: i1, .. }, KeybindingChange::Modify { index: i2, .. }) => i1.cmp(i2),
            (KeybindingChange::Modify { .. }, _) => std::cmp::Ordering::Less,
            (_, KeybindingChange::Modify { .. }) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    for change in sorted_changes {
        match change {
            KeybindingChange::Delete(index) => {
                if *index < children.nodes().len() {
                    children.nodes_mut().remove(*index);
                }
            }
            KeybindingChange::Modify { index, new } => {
                if *index < children.nodes().len() {
                    let node = create_keybinding_node(new);
                    children.nodes_mut()[*index] = node;
                }
            }
            KeybindingChange::Add(binding) => {
                let node = create_keybinding_node(binding);
                children.nodes_mut().push(node);
            }
        }
    }

    // Autoformat the binds block
    children.autoformat();
    binds_node.autoformat();

    config.save()
}

/// Create a KDL node for a keybinding
fn create_keybinding_node(binding: &Keybinding) -> KdlNode {
    let combo = binding.combo();
    let mut node = KdlNode::new(combo);

    // Add properties
    if let Some(repeat) = binding.properties.repeat {
        node.push(KdlEntry::new_prop("repeat", KdlValue::Bool(repeat)));
    }
    if let Some(cooldown) = binding.properties.cooldown_ms {
        node.push(KdlEntry::new_prop("cooldown-ms", KdlValue::Integer(cooldown as i128)));
    }
    if let Some(allow_locked) = binding.properties.allow_when_locked {
        node.push(KdlEntry::new_prop("allow-when-locked", KdlValue::Bool(allow_locked)));
    }

    // Create action child node
    let mut children = KdlDocument::new();
    let action_node = create_action_node(&binding.action);
    children.nodes_mut().push(action_node);
    children.autoformat();

    node.set_children(children);
    node.autoformat();

    node
}

/// Create a KDL node for an action
fn create_action_node(action: &BindingAction) -> KdlNode {
    match action {
        BindingAction::Spawn(args) => {
            let mut node = KdlNode::new("spawn");
            for arg in args {
                node.push(KdlEntry::new(KdlValue::String(arg.clone())));
            }
            node.autoformat();
            node
        }
        BindingAction::SpawnSh(cmd) => {
            let mut node = KdlNode::new("spawn-sh");
            node.push(KdlEntry::new(KdlValue::String(cmd.clone())));
            node.autoformat();
            node
        }
        BindingAction::Simple(name) => {
            let mut node = KdlNode::new(name.as_str());
            node.autoformat();
            node
        }
        BindingAction::WithArg(name, arg) => {
            let mut node = KdlNode::new(name.as_str());
            let value = match arg {
                BindingArg::Number(n) => KdlValue::Integer(*n as i128),
                BindingArg::String(s) => KdlValue::String(s.clone()),
                BindingArg::Bool(b) => KdlValue::Bool(*b),
            };
            node.push(KdlEntry::new(value));
            node.autoformat();
            node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BindingProperties, Modifiers};

    #[test]
    fn test_create_keybinding_node_simple() {
        let binding = Keybinding {
            modifiers: Modifiers {
                mod_key: true,
                ctrl: false,
                shift: false,
                alt: false,
            },
            key: "Q".to_string(),
            properties: BindingProperties::default(),
            action: BindingAction::Simple("close-window".to_string()),
            kdl_index: None,
        };

        let node = create_keybinding_node(&binding);
        assert_eq!(node.name().value(), "Mod+Q");
    }

    #[test]
    fn test_create_keybinding_node_with_properties() {
        let binding = Keybinding {
            modifiers: Modifiers {
                mod_key: true,
                ctrl: false,
                shift: false,
                alt: false,
            },
            key: "Q".to_string(),
            properties: BindingProperties {
                repeat: Some(false),
                cooldown_ms: None,
                allow_when_locked: None,
            },
            action: BindingAction::Simple("close-window".to_string()),
            kdl_index: None,
        };

        let node = create_keybinding_node(&binding);
        assert!(node.get("repeat").is_some());
    }
}
