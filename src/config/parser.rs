use anyhow::Result;
use std::path::PathBuf;

use crate::model::{ConfigDocument, Position};

/// Load and parse the niri config file
pub fn load_config() -> Result<ConfigDocument> {
    let path = get_config_path()?;
    ConfigDocument::load(path)
}

/// Get the default niri config path
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    Ok(config_dir.join("niri").join("config.kdl"))
}

/// Extract output positions from config
pub fn get_configured_positions(config: &ConfigDocument) -> Vec<(String, Position)> {
    let mut positions = Vec::new();

    for node in config.doc.nodes() {
        let name_value = node.name().value();
        if name_value == "output" || name_value == "/-output" {
            if let Some(output_name) = node.get(0).and_then(|v| v.as_string()) {
                if let Some(pos) = config.get_output_position(output_name) {
                    positions.push((output_name.to_string(), pos));
                }
            }
        }
    }

    positions
}
