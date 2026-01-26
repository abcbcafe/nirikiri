use anyhow::{Context, Result};
use kdl::{KdlDocument, KdlNode, KdlEntry, KdlValue};
use std::path::PathBuf;

use super::output::Position;

/// Wrapper around KdlDocument that preserves formatting
pub struct ConfigDocument {
    pub doc: KdlDocument,
    pub path: PathBuf,
}

impl ConfigDocument {
    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        // niri uses KDL v1 syntax, so parse explicitly as v1
        let doc = KdlDocument::parse_v1(&content)
            .with_context(|| format!("Failed to parse KDL config from {}", path.display()))?;
        Ok(Self { doc, path })
    }

    pub fn save(&mut self) -> Result<()> {
        // Create backup first
        let backup_path = self.path.with_extension("kdl.bak");
        if self.path.exists() {
            std::fs::copy(&self.path, &backup_path)
                .with_context(|| "Failed to create config backup")?;
        }

        // Ensure v1 format for niri compatibility
        self.doc.ensure_v1();

        std::fs::write(&self.path, self.doc.to_string())
            .with_context(|| "Failed to write config file")?;
        Ok(())
    }

    /// Find an output node by name (including commented-out nodes with /-)
    pub fn find_output_node(&self, name: &str) -> Option<(usize, bool)> {
        for (idx, node) in self.doc.nodes().iter().enumerate() {
            // Check for normal output node
            if node.name().value() == "output" {
                if let Some(output_name) = node.get(0) {
                    if output_name.as_string() == Some(name) {
                        return Some((idx, false));
                    }
                }
            }
            // Check for commented output node (starts with /-)
            if node.name().value() == "/-output" {
                if let Some(output_name) = node.get(0) {
                    if output_name.as_string() == Some(name) {
                        return Some((idx, true));
                    }
                }
            }
        }
        None
    }

    /// Get position from an output node
    pub fn get_output_position(&self, name: &str) -> Option<Position> {
        let (idx, _commented) = self.find_output_node(name)?;
        let node = self.doc.nodes().get(idx)?;
        let children = node.children()?;

        for child in children.nodes() {
            if child.name().value() == "position" {
                let x = child
                    .get("x")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as i32;
                let y = child
                    .get("y")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as i32;
                return Some(Position::new(x, y));
            }
        }
        None
    }

    /// Update or create position for an output
    pub fn set_output_position(&mut self, name: &str, position: Position) -> Result<()> {
        if let Some((idx, commented)) = self.find_output_node(name) {
            // Get mutable access to the node
            let node = self.doc.nodes_mut().get_mut(idx).unwrap();

            // If commented, uncomment it first
            if commented {
                node.set_name("output");
            }

            // Ensure children exist
            if node.children().is_none() {
                node.set_children(KdlDocument::new());
            }

            let children = node.children_mut().as_mut().unwrap();

            // Find or create position node
            let position_idx = children
                .nodes()
                .iter()
                .position(|n| n.name().value() == "position");

            if let Some(pos_idx) = position_idx {
                // Update existing position node
                let pos_node = children.nodes_mut().get_mut(pos_idx).unwrap();
                pos_node.entries_mut().clear();
                pos_node.push(KdlEntry::new_prop("x", KdlValue::Integer(position.x as i128)));
                pos_node.push(KdlEntry::new_prop("y", KdlValue::Integer(position.y as i128)));
                pos_node.autoformat();
            } else {
                // Create new position node
                let mut pos_node = KdlNode::new("position");
                pos_node.push(KdlEntry::new_prop("x", KdlValue::Integer(position.x as i128)));
                pos_node.push(KdlEntry::new_prop("y", KdlValue::Integer(position.y as i128)));
                pos_node.autoformat();
                children.nodes_mut().push(pos_node);
            }
        } else {
            // Create new output node with proper formatting
            let mut output_node = KdlNode::new("output");
            output_node.push(KdlEntry::new(KdlValue::String(name.to_string())));

            let mut children = KdlDocument::new();
            let mut pos_node = KdlNode::new("position");
            pos_node.push(KdlEntry::new_prop("x", KdlValue::Integer(position.x as i128)));
            pos_node.push(KdlEntry::new_prop("y", KdlValue::Integer(position.y as i128)));
            pos_node.autoformat();
            children.nodes_mut().push(pos_node);
            children.autoformat();

            output_node.set_children(children);
            output_node.autoformat();
            self.doc.nodes_mut().push(output_node);
        }
        Ok(())
    }
}

