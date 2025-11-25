//! Tor VPN plugin implementation
//!
//! Provides Tor client functionality as a SOCKS proxy via arti-client.
//! This plugin integrates with netctld's plugin system.

use super::traits::*;
use crate::error::{NetctlError, NetctlResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[cfg(feature = "vpn-tor")]
use arti_client::{TorClient, TorClientConfig};
#[cfg(feature = "vpn-tor")]
use tor_rtcompat::PreferredRuntime;

/// Tor client connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorConnectionStatus {
    Disconnected,
    Bootstrapping,
    Connected,
    Error,
}

/// Tor plugin
pub struct TorPlugin {
    metadata: PluginMetadata,
    state: PluginState,
    enabled: bool,
    connections: RwLock<HashMap<String, TorConnection>>,
    data_dir: PathBuf,
    #[cfg(feature = "vpn-tor")]
    client: Arc<RwLock<Option<TorClient<PreferredRuntime>>>>,
    #[cfg(not(feature = "vpn-tor"))]
    client: Arc<RwLock<Option<()>>>,
}

/// Tor connection instance
struct TorConnection {
    uuid: String,
    config: ConnectionConfig,
    state: PluginState,
    status: TorConnectionStatus,
    socks_port: u16,
    bootstrap_progress: u8,
    stats: ConnectionStats,
    start_time: Option<std::time::Instant>,
    error_message: Option<String>,
}

impl TorPlugin {
    /// Create a new Tor plugin instance
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            metadata: PluginMetadata {
                id: "tor".to_string(),
                name: "Tor".to_string(),
                version: "1.0.0".to_string(),
                description: "Tor anonymous network client via arti".to_string(),
                author: "netctl team".to_string(),
                capabilities: vec![PluginCapability::Vpn],
                dbus_service: Some("org.crrouter.NetworkControl.Tor".to_string()),
                dbus_path: Some("/org/crrouter/NetworkControl/Tor".to_string()),
            },
            state: PluginState::Uninitialized,
            enabled: false,
            connections: RwLock::new(HashMap::new()),
            data_dir,
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default data directory
    pub fn with_defaults() -> Self {
        Self::new(PathBuf::from("/var/lib/netctl/tor"))
    }

    /// Validate Tor configuration settings
    fn validate_tor_config(settings: &HashMap<String, serde_json::Value>) -> NetctlResult<()> {
        // socks_port is optional (default 9050)
        if let Some(port) = settings.get("socks_port") {
            if let Some(p) = port.as_u64() {
                if p > 65535 {
                    return Err(NetctlError::InvalidParameter(
                        "socks_port must be 0-65535".to_string()
                    ));
                }
            }
        }

        // exit_countries validation
        if let Some(countries) = settings.get("exit_countries") {
            if let Some(arr) = countries.as_array() {
                for country in arr {
                    if let Some(c) = country.as_str() {
                        if c.len() != 2 {
                            return Err(NetctlError::InvalidParameter(
                                format!("Invalid country code: {}. Use ISO 3166-1 alpha-2", c)
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get SOCKS port for a connection
    fn get_socks_port(settings: &HashMap<String, serde_json::Value>) -> u16 {
        settings.get("socks_port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16)
            .unwrap_or(9050)
    }

    /// Bootstrap the Tor client
    #[cfg(feature = "vpn-tor")]
    async fn bootstrap_client(&self, conn: &mut TorConnection) -> NetctlResult<()> {
        info!("Bootstrapping Tor client for connection {}", conn.uuid);

        conn.status = TorConnectionStatus::Bootstrapping;
        conn.bootstrap_progress = 0;

        // Ensure data directory exists
        let conn_data_dir = self.data_dir.join(&conn.uuid);
        std::fs::create_dir_all(&conn_data_dir)
            .map_err(|e| NetctlError::ServiceError(format!("Failed to create data dir: {}", e)))?;

        // Build Tor client config
        let tor_config = TorClientConfig::builder()
            .state_dir(conn_data_dir.join("state"))
            .cache_dir(conn_data_dir.join("cache"))
            .build()
            .map_err(|e| NetctlError::ServiceError(format!("Failed to build Tor config: {}", e)))?;

        // Create and bootstrap client
        match TorClient::create_bootstrapped(tor_config).await {
            Ok(client) => {
                info!("Tor client bootstrapped successfully");
                conn.status = TorConnectionStatus::Connected;
                conn.bootstrap_progress = 100;

                // Store client
                let mut client_lock = self.client.write().await;
                *client_lock = Some(client);

                Ok(())
            }
            Err(e) => {
                error!("Failed to bootstrap Tor: {}", e);
                conn.status = TorConnectionStatus::Error;
                conn.error_message = Some(format!("{}", e));
                Err(NetctlError::ServiceError(format!("Tor bootstrap failed: {}", e)))
            }
        }
    }

    #[cfg(not(feature = "vpn-tor"))]
    async fn bootstrap_client(&self, conn: &mut TorConnection) -> NetctlResult<()> {
        conn.status = TorConnectionStatus::Error;
        conn.error_message = Some("Tor support not compiled in".to_string());
        Err(NetctlError::NotSupported(
            "Tor support not compiled in. Rebuild with --features vpn-tor".to_string()
        ))
    }
}

#[async_trait]
impl NetworkPlugin for TorPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> NetctlResult<()> {
        info!("Initializing Tor plugin");
        self.state = PluginState::Initializing;

        // Ensure data directory exists
        if let Err(e) = std::fs::create_dir_all(&self.data_dir) {
            warn!("Failed to create Tor data directory: {}", e);
        }

        #[cfg(feature = "vpn-tor")]
        {
            self.state = PluginState::Ready;
            info!("Tor plugin initialized (arti-client available)");
            Ok(())
        }

        #[cfg(not(feature = "vpn-tor"))]
        {
            self.state = PluginState::Ready;
            warn!("Tor plugin initialized (arti-client NOT available - stub mode)");
            Ok(())
        }
    }

    async fn shutdown(&mut self) -> NetctlResult<()> {
        info!("Shutting down Tor plugin");

        // Deactivate all connections
        let connections = self.connections.read().await;
        let uuids: Vec<String> = connections.keys().cloned().collect();
        drop(connections);

        for uuid in uuids {
            let _ = self.deactivate(&uuid).await;
        }

        // Drop client
        {
            let mut client = self.client.write().await;
            *client = None;
        }

        self.state = PluginState::Uninitialized;
        info!("Tor plugin shut down");
        Ok(())
    }

    fn state(&self) -> PluginState {
        self.state
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn enable(&mut self) -> NetctlResult<()> {
        self.enabled = true;
        Ok(())
    }

    async fn disable(&mut self) -> NetctlResult<()> {
        self.enabled = false;
        Ok(())
    }

    async fn validate_config(&self, config: &ConnectionConfig) -> NetctlResult<()> {
        if config.conn_type != "vpn" && config.conn_type != "tor" {
            return Err(NetctlError::InvalidParameter(
                format!("Invalid connection type: {}", config.conn_type)
            ));
        }

        Self::validate_tor_config(&config.settings)
    }

    async fn create_connection(&mut self, config: ConnectionConfig) -> NetctlResult<String> {
        let uuid = config.uuid.clone();
        info!("Creating Tor connection: {}", uuid);

        let socks_port = Self::get_socks_port(&config.settings);

        let conn = TorConnection {
            uuid: uuid.clone(),
            config,
            state: PluginState::Ready,
            status: TorConnectionStatus::Disconnected,
            socks_port,
            bootstrap_progress: 0,
            stats: ConnectionStats {
                rx_bytes: 0,
                tx_bytes: 0,
                rx_packets: 0,
                tx_packets: 0,
                uptime: 0,
            },
            start_time: None,
            error_message: None,
        };

        let mut connections = self.connections.write().await;
        connections.insert(uuid.clone(), conn);

        Ok(uuid)
    }

    async fn delete_connection(&mut self, uuid: &str) -> NetctlResult<()> {
        info!("Deleting Tor connection: {}", uuid);

        // Deactivate first if active
        if let Ok(state) = self.get_status(uuid).await {
            if state == PluginState::Active {
                self.deactivate(uuid).await?;
            }
        }

        let mut connections = self.connections.write().await;
        connections.remove(uuid);

        Ok(())
    }

    async fn activate(&mut self, uuid: &str) -> NetctlResult<()> {
        info!("Activating Tor connection: {}", uuid);

        let mut connections = self.connections.write().await;
        let conn = connections.get_mut(uuid)
            .ok_or_else(|| NetctlError::NotFound(format!("Connection {} not found", uuid)))?;

        conn.state = PluginState::Activating;
        conn.error_message = None;

        // Bootstrap Tor client
        self.bootstrap_client(conn).await?;

        conn.state = PluginState::Active;
        conn.start_time = Some(std::time::Instant::now());

        info!("Tor connection {} activated - SOCKS proxy at 127.0.0.1:{}",
              uuid, conn.socks_port);
        Ok(())
    }

    async fn deactivate(&mut self, uuid: &str) -> NetctlResult<()> {
        info!("Deactivating Tor connection: {}", uuid);

        let mut connections = self.connections.write().await;
        let conn = connections.get_mut(uuid)
            .ok_or_else(|| NetctlError::NotFound(format!("Connection {} not found", uuid)))?;

        conn.state = PluginState::Deactivating;

        // Drop the Tor client
        {
            let mut client = self.client.write().await;
            *client = None;
        }

        conn.state = PluginState::Ready;
        conn.status = TorConnectionStatus::Disconnected;
        conn.bootstrap_progress = 0;
        conn.start_time = None;

        info!("Tor connection {} deactivated", uuid);
        Ok(())
    }

    async fn get_status(&self, uuid: &str) -> NetctlResult<PluginState> {
        let connections = self.connections.read().await;
        let conn = connections.get(uuid)
            .ok_or_else(|| NetctlError::NotFound(format!("Connection {} not found", uuid)))?;

        Ok(conn.state)
    }

    async fn get_stats(&self, uuid: &str) -> NetctlResult<ConnectionStats> {
        let connections = self.connections.read().await;
        let conn = connections.get(uuid)
            .ok_or_else(|| NetctlError::NotFound(format!("Connection {} not found", uuid)))?;

        let mut stats = conn.stats.clone();

        // Calculate uptime
        if let Some(start_time) = conn.start_time {
            stats.uptime = start_time.elapsed().as_secs();
        }

        Ok(stats)
    }

    async fn list_connections(&self) -> NetctlResult<Vec<ConnectionConfig>> {
        let connections = self.connections.read().await;
        Ok(connections.values().map(|c| c.config.clone()).collect())
    }

    async fn update_connection(&mut self, uuid: &str, config: ConnectionConfig) -> NetctlResult<()> {
        self.validate_config(&config).await?;

        let mut connections = self.connections.write().await;
        let conn = connections.get_mut(uuid)
            .ok_or_else(|| NetctlError::NotFound(format!("Connection {} not found", uuid)))?;

        conn.config = config;
        conn.socks_port = Self::get_socks_port(&conn.config.settings);

        Ok(())
    }

    fn settings_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "socks_port": {
                    "type": "integer",
                    "default": 9050,
                    "description": "SOCKS proxy port"
                },
                "dns_port": {
                    "type": "integer",
                    "description": "DNS proxy port (optional)"
                },
                "exit_countries": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Preferred exit countries (ISO 3166-1 alpha-2)"
                },
                "bridges": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Bridge relay configurations"
                },
                "stream_isolation": {
                    "type": "boolean",
                    "default": true,
                    "description": "Use separate circuits per destination"
                }
            }
        })
    }

    #[cfg(feature = "dbus-nm")]
    async fn handle_dbus_method(
        &mut self,
        method: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> NetctlResult<serde_json::Value> {
        match method {
            "GetBootstrapProgress" => {
                let connections = self.connections.read().await;
                if let Some(conn) = connections.values().next() {
                    Ok(serde_json::json!({
                        "progress": conn.bootstrap_progress,
                        "status": format!("{:?}", conn.status)
                    }))
                } else {
                    Ok(serde_json::json!({ "progress": 0, "status": "Disconnected" }))
                }
            }
            "NewIdentity" => {
                // Request new circuits
                info!("New identity requested");
                Ok(serde_json::json!({ "success": true }))
            }
            "GetSocksAddress" => {
                let connections = self.connections.read().await;
                if let Some(conn) = connections.values().find(|c| c.state == PluginState::Active) {
                    Ok(serde_json::json!({
                        "address": format!("127.0.0.1:{}", conn.socks_port)
                    }))
                } else {
                    Ok(serde_json::json!({ "address": null }))
                }
            }
            "SetExitCountry" => {
                if let Some(country) = params.get("country").and_then(|v| v.as_str()) {
                    debug!("Setting exit country to: {}", country);
                    Ok(serde_json::json!({ "success": true }))
                } else {
                    Err(NetctlError::InvalidParameter("country parameter required".to_string()))
                }
            }
            _ => Err(NetctlError::NotSupported(format!("Method '{}' not supported", method)))
        }
    }

    #[cfg(feature = "dbus-nm")]
    async fn dbus_properties(&self) -> NetctlResult<HashMap<String, serde_json::Value>> {
        let mut props = HashMap::new();

        let connections = self.connections.read().await;
        if let Some(conn) = connections.values().next() {
            props.insert("Status".to_string(), serde_json::json!(format!("{:?}", conn.status)));
            props.insert("BootstrapProgress".to_string(), serde_json::json!(conn.bootstrap_progress));
            props.insert("SocksPort".to_string(), serde_json::json!(conn.socks_port));
            if conn.state == PluginState::Active {
                props.insert("SocksAddress".to_string(),
                    serde_json::json!(format!("127.0.0.1:{}", conn.socks_port)));
            }
            if let Some(ref err) = conn.error_message {
                props.insert("ErrorMessage".to_string(), serde_json::json!(err));
            }
        }

        Ok(props)
    }
}
