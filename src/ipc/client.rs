use anyhow::{Context, Result, bail};
use niri_ipc::{socket::Socket, Request, Response, Output, OutputConfigChanged, ConfiguredPosition, PositionToSet, Action};

use crate::model::{OutputMode, OutputState, OutputTransform, Position, Size};

/// Client wrapper for niri IPC
pub struct NiriClient {
    socket: Socket,
}

impl NiriClient {
    pub fn connect() -> Result<Self> {
        let socket = Socket::connect().context("Failed to connect to niri socket. Is niri running?")?;
        Ok(Self { socket })
    }

    /// Query all outputs from niri
    pub fn get_outputs(&mut self) -> Result<Vec<OutputState>> {
        let reply = self.socket.send(Request::Outputs).context("Failed to send Outputs request")?;
        let response = reply.map_err(|e| anyhow::anyhow!("niri error: {e}"))?;

        match response {
            Response::Outputs(outputs) => {
                outputs
                    .into_values()
                    .map(|o| self.convert_output(o))
                    .collect()
            }
            other => bail!("Unexpected response: {other:?}"),
        }
    }

    fn convert_output(&self, output: Output) -> Result<OutputState> {
        let modes: Vec<OutputMode> = output
            .modes
            .iter()
            .map(|m| OutputMode {
                width: m.width as u32,
                height: m.height as u32,
                refresh_rate: m.refresh_rate as f64 / 1000.0,
                is_preferred: m.is_preferred,
            })
            .collect();

        let current_mode_index = output.current_mode;

        // Get logical info if available
        let (position, logical_size, scale, transform, enabled) = if let Some(logical) = &output.logical {
            (
                Position::new(logical.x, logical.y),
                Size::new(logical.width, logical.height),
                logical.scale,
                OutputTransform::from_niri(&logical.transform),
                true,
            )
        } else {
            (
                Position::default(),
                Size::default(),
                1.0,
                OutputTransform::Normal,
                false,
            )
        };

        let physical_size = output
            .current_mode
            .and_then(|idx| output.modes.get(idx))
            .map(|m| Size::new(m.width as u32, m.height as u32))
            .unwrap_or_default();

        Ok(OutputState {
            name: output.name,
            modes,
            current_mode_index,
            scale,
            transform,
            position,
            logical_size,
            physical_size,
            enabled,
            connected: true, // If we get it from IPC, it's connected
            configured: false, // Will be set later when merging with config
            make: output.make,
            model: output.model,
        })
    }

    /// Reload niri config
    pub fn reload_config(&mut self) -> Result<()> {
        let reply = self.socket.send(Request::Action(Action::LoadConfigFile {}))
            .context("Failed to send LoadConfigFile request")?;
        reply.map_err(|e| anyhow::anyhow!("niri error: {e}"))?;
        Ok(())
    }

    /// Preview output position change via IPC
    pub fn preview_position(&mut self, name: &str, position: Position) -> Result<OutputConfigChanged> {
        let action = niri_ipc::OutputAction::Position {
            position: PositionToSet::Specific(ConfiguredPosition {
                x: position.x,
                y: position.y,
            }),
        };

        let request = Request::Output {
            output: name.to_string(),
            action,
        };

        let reply = self.socket.send(request).context("Failed to send Output request")?;
        let response = reply.map_err(|e| anyhow::anyhow!("niri error: {e}"))?;

        match response {
            Response::OutputConfigChanged(changed) => Ok(changed),
            other => bail!("Unexpected response: {other:?}"),
        }
    }
}

