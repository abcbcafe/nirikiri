use anyhow::Result;
use std::collections::HashMap;

use crate::model::{ConfigDocument, Position};

/// Write pending position changes to the config
pub fn write_positions(
    config: &mut ConfigDocument,
    positions: &HashMap<String, Position>,
) -> Result<()> {
    for (name, position) in positions {
        config.set_output_position(name, *position)?;
    }
    config.save()
}
