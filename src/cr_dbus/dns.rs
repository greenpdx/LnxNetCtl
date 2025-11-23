//! CR DNS Server D-Bus interface
//!
//! D-Bus interface for DNS server management

use super::types::*;
use crate::error::{NetctlError, NetctlResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use zbus::{Connection, fdo, interface};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

/// CR DNS Server D-Bus interface
#[derive(Clone)]
pub struct CRDns {
    /// Whether DNS server is running
    running: Arc<RwLock<bool>>,
    /// DNS forwarders (upstream DNS servers)
    forwarders: Arc<RwLock<Vec<String>>>,
    /// Listen address
    listen_address: Arc<RwLock<Option<String>>>,
    /// Listen port
    listen_port: Arc<RwLock<u16>>,
}

impl CRDns {
    /// Create a new CR DNS interface
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            forwarders: Arc::new(RwLock::new(Vec::new())),
            listen_address: Arc::new(RwLock::new(None)),
            listen_port: Arc::new(RwLock::new(53)),
        }
    }

    /// Set running state
    pub async fn set_running(&self, running: bool) {
        let mut r = self.running.write().await;
        *r = running;
        info!("CR DNS: Server running state set to {}", running);
    }

    /// Add a forwarder
    pub async fn add_forwarder_internal(&self, forwarder: String) {
        let mut forwarders = self.forwarders.write().await;
        if !forwarders.contains(&forwarder) {
            forwarders.push(forwarder);
        }
    }

    /// Remove a forwarder
    pub async fn remove_forwarder_internal(&self, forwarder: &str) -> bool {
        let mut forwarders = self.forwarders.write().await;
        if let Some(pos) = forwarders.iter().position(|f| f == forwarder) {
            forwarders.remove(pos);
            true
        } else {
            false
        }
    }
}

#[interface(name = "org.crrouter.NetworkControl.DNS")]
impl CRDns {
    /// Start DNS server
    async fn start_server(
        &self,
        listen_address: &str,
        listen_port: u16,
        forwarders: Vec<String>,
    ) -> fdo::Result<()> {
        info!(
            "CR DNS: Starting DNS server on {}:{}",
            listen_address, listen_port
        );

        // Validate parameters
        if listen_address.is_empty() {
            return Err(fdo::Error::InvalidArgs("Listen address cannot be empty".to_string()));
        }

        if listen_port == 0 {
            return Err(fdo::Error::InvalidArgs("Listen port cannot be 0".to_string()));
        }

        // Check if already running
        let running = self.running.read().await;
        if *running {
            return Err(fdo::Error::Failed("DNS server already running".to_string()));
        }
        drop(running);

        // Set configuration
        let mut addr = self.listen_address.write().await;
        *addr = Some(listen_address.to_string());
        drop(addr);

        let mut port = self.listen_port.write().await;
        *port = listen_port;
        drop(port);

        // Set forwarders
        let mut fwd = self.forwarders.write().await;
        *fwd = forwarders;
        drop(fwd);

        // Set running state
        self.set_running(true).await;

        // Actual DNS server start will be handled by integration layer

        Ok(())
    }

    /// Stop DNS server
    async fn stop_server(&self) -> fdo::Result<()> {
        info!("CR DNS: Stopping DNS server");

        let running = self.running.read().await;
        if !*running {
            return Err(fdo::Error::Failed("DNS server not running".to_string()));
        }
        drop(running);

        // Clear configuration
        let mut addr = self.listen_address.write().await;
        *addr = None;
        drop(addr);

        // Set running state
        self.set_running(false).await;

        // Actual DNS server stop will be handled by integration layer

        Ok(())
    }

    /// Add a DNS forwarder
    async fn add_forwarder(&self, forwarder: &str) -> fdo::Result<()> {
        info!("CR DNS: Adding forwarder: {}", forwarder);

        if forwarder.is_empty() {
            return Err(fdo::Error::InvalidArgs("Forwarder address cannot be empty".to_string()));
        }

        // Basic validation - check if it's a valid IP address format
        if !forwarder.contains('.') && !forwarder.contains(':') {
            return Err(fdo::Error::InvalidArgs("Invalid forwarder address format".to_string()));
        }

        self.add_forwarder_internal(forwarder.to_string()).await;

        Ok(())
    }

    /// Remove a DNS forwarder
    async fn remove_forwarder(&self, forwarder: &str) -> fdo::Result<()> {
        info!("CR DNS: Removing forwarder: {}", forwarder);

        if !self.remove_forwarder_internal(forwarder).await {
            return Err(fdo::Error::Failed(format!("Forwarder not found: {}", forwarder)));
        }

        Ok(())
    }

    /// Get all DNS forwarders
    async fn get_forwarders(&self) -> Vec<String> {
        let forwarders = self.forwarders.read().await;
        debug!("CR DNS: Returning {} forwarders", forwarders.len());
        forwarders.clone()
    }

    /// Get DNS server status
    async fn get_status(&self) -> HashMap<String, Value<'static>> {
        let mut status = HashMap::new();

        let running = self.running.read().await;
        status.insert("Running".to_string(), Value::new(*running));

        if let Some(ref addr) = *self.listen_address.read().await {
            status.insert("ListenAddress".to_string(), Value::new(addr.clone()));
        }

        let port = *self.listen_port.read().await;
        status.insert("ListenPort".to_string(), Value::new(port));

        let forwarders = self.forwarders.read().await;
        status.insert("Forwarders".to_string(), Value::new(forwarders.clone()));
        status.insert("ForwarderCount".to_string(), Value::new(forwarders.len() as u32));

        debug!("CR DNS: Returning status");
        status
    }

    /// Check if DNS server is running
    async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Set DNS forwarders (replaces all existing forwarders)
    async fn set_forwarders(&self, forwarders: Vec<String>) -> fdo::Result<()> {
        info!("CR DNS: Setting {} forwarders", forwarders.len());

        // Validate all forwarders
        for forwarder in &forwarders {
            if forwarder.is_empty() {
                return Err(fdo::Error::InvalidArgs("Forwarder address cannot be empty".to_string()));
            }
            if !forwarder.contains('.') && !forwarder.contains(':') {
                return Err(fdo::Error::InvalidArgs(format!("Invalid forwarder address: {}", forwarder)));
            }
        }

        let mut fwd = self.forwarders.write().await;
        *fwd = forwarders;

        Ok(())
    }

    // ============ D-Bus Signals ============

    /// ServerStarted signal - emitted when DNS server starts
    #[zbus(signal)]
    async fn server_started(
        signal_emitter: &SignalEmitter<'_>,
        listen_address: &str,
        listen_port: u16,
    ) -> zbus::Result<()>;

    /// ServerStopped signal - emitted when DNS server stops
    #[zbus(signal)]
    async fn server_stopped(signal_emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// ForwarderAdded signal - emitted when a forwarder is added
    #[zbus(signal)]
    async fn forwarder_added(
        signal_emitter: &SignalEmitter<'_>,
        forwarder: &str,
    ) -> zbus::Result<()>;

    /// ForwarderRemoved signal - emitted when a forwarder is removed
    #[zbus(signal)]
    async fn forwarder_removed(
        signal_emitter: &SignalEmitter<'_>,
        forwarder: &str,
    ) -> zbus::Result<()>;
}

impl Default for CRDns {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper module for emitting DNS signals
pub mod signals {
    use super::*;

    /// Emit ServerStarted signal
    pub async fn emit_server_started(
        conn: &Connection,
        listen_address: &str,
        listen_port: u16,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRDns>(CR_DNS_PATH)
            .await
        {
            CRDns::server_started(iface_ref.signal_emitter(), listen_address, listen_port)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ServerStarted: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ServerStopped signal
    pub async fn emit_server_stopped(conn: &Connection) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRDns>(CR_DNS_PATH)
            .await
        {
            CRDns::server_stopped(iface_ref.signal_emitter())
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ServerStopped: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ForwarderAdded signal
    pub async fn emit_forwarder_added(
        conn: &Connection,
        forwarder: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRDns>(CR_DNS_PATH)
            .await
        {
            CRDns::forwarder_added(iface_ref.signal_emitter(), forwarder)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ForwarderAdded: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ForwarderRemoved signal
    pub async fn emit_forwarder_removed(
        conn: &Connection,
        forwarder: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRDns>(CR_DNS_PATH)
            .await
        {
            CRDns::forwarder_removed(iface_ref.signal_emitter(), forwarder)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ForwarderRemoved: {}", e)))?;
        }
        Ok(())
    }
}
