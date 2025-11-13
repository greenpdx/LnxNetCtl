//! Routing table management

use crate::error::{NetctlError, NetctlResult};
use tokio::process::Command;

pub struct RoutingController;

impl RoutingController {
    pub fn new() -> Self {
        Self
    }

    pub async fn add_default_gateway(&self, gateway: &str, interface: Option<&str>) -> NetctlResult<()> {
        let mut args = vec!["route", "add", "default", "via", gateway];
        if let Some(iface) = interface {
            args.extend_from_slice(&["dev", iface]);
        }

        let cmd_str = format!("ip {}", args.join(" "));
        let output = Command::new("ip")
            .args(&args)
            .output()
            .await
            .map_err(|e| NetctlError::CommandFailed {
                cmd: cmd_str.clone(),
                code: None,
                stderr: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).to_string());
            return Err(NetctlError::CommandFailed {
                cmd: cmd_str,
                code: output.status.code(),
                stderr,
            });
        }
        Ok(())
    }
}

impl Default for RoutingController {
    fn default() -> Self {
        Self::new()
    }
}
