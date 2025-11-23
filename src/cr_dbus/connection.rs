//! CR Connection D-Bus interface
//!
//! D-Bus interface for network connection management

use super::types::*;
use crate::error::{NetctlError, NetctlResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use zbus::{Connection, fdo, interface};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;
use uuid::Uuid;

/// CR Connection D-Bus interface
#[derive(Clone)]
pub struct CRConnection {
    /// All network connections by UUID
    connections: Arc<RwLock<HashMap<String, CRConnectionInfo>>>,
}

impl CRConnection {
    /// Create a new CR Connection interface
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a connection
    pub async fn add_connection_internal(&self, conn_info: CRConnectionInfo) {
        let mut connections = self.connections.write().await;
        let uuid = conn_info.uuid.clone();
        info!("CR Connection: Adding connection {} ({})", conn_info.id, uuid);
        connections.insert(uuid, conn_info);
    }

    /// Remove a connection
    pub async fn remove_connection_internal(&self, uuid: &str) -> NetctlResult<()> {
        let mut connections = self.connections.write().await;
        if connections.remove(uuid).is_some() {
            info!("CR Connection: Removed connection {}", uuid);
            Ok(())
        } else {
            Err(NetctlError::NotFound(format!("Connection {} not found", uuid)))
        }
    }

    /// Update connection state
    pub async fn update_state(&self, uuid: &str, state: CRConnectionState) -> NetctlResult<()> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(uuid) {
            conn.state = state;
            info!("CR Connection: Connection {} state changed to {:?}", uuid, state);
            Ok(())
        } else {
            Err(NetctlError::NotFound(format!("Connection {} not found", uuid)))
        }
    }

    /// Get connection info
    pub async fn get_connection_info(&self, uuid: &str) -> Option<CRConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(uuid).cloned()
    }
}

#[interface(name = "org.crrouter.NetworkControl.Connection")]
impl CRConnection {
    /// List all connections
    async fn list_connections(&self) -> Vec<HashMap<String, Value<'static>>> {
        let connections = self.connections.read().await;
        let mut result = Vec::new();

        for (_, conn) in connections.iter() {
            let mut conn_info = HashMap::new();
            conn_info.insert("UUID".to_string(), Value::new(conn.uuid.clone()));
            conn_info.insert("ID".to_string(), Value::new(conn.id.clone()));
            conn_info.insert("Type".to_string(), Value::new(conn.conn_type as u32));
            conn_info.insert("State".to_string(), Value::new(conn.state as u32));
            conn_info.insert("Autoconnect".to_string(), Value::new(conn.autoconnect));

            if let Some(ref device) = conn.device {
                conn_info.insert("Device".to_string(), Value::new(device.clone()));
            }

            result.push(conn_info);
        }

        debug!("CR Connection: Returning {} connections", result.len());
        result
    }

    /// Get connection details by UUID or ID
    async fn get_connection(&self, id: &str) -> fdo::Result<HashMap<String, Value<'static>>> {
        let connections = self.connections.read().await;

        // Try to find by UUID first
        if let Some(conn) = connections.get(id) {
            return Ok(self.connection_to_hashmap(conn));
        }

        // Try to find by ID (name)
        for (_, conn) in connections.iter() {
            if conn.id == id {
                return Ok(self.connection_to_hashmap(conn));
            }
        }

        Err(fdo::Error::Failed(format!("Connection {} not found", id)))
    }

    /// Add a new connection
    async fn add_connection(&self, settings: HashMap<String, Value<'_>>) -> fdo::Result<String> {
        // Extract required fields
        let id = settings.get("ID")
            .ok_or_else(|| fdo::Error::InvalidArgs("Missing ID field".to_string()))?
            .downcast_ref::<&str>()
            .map_err(|e| fdo::Error::InvalidArgs(format!("Invalid ID type: {}", e)))?
            .to_string();

        let conn_type_u32 = settings.get("Type")
            .ok_or_else(|| fdo::Error::InvalidArgs("Missing Type field".to_string()))?
            .downcast_ref::<u32>()
            .map_err(|e| fdo::Error::InvalidArgs(format!("Invalid Type: {}", e)))?;

        let conn_type = match conn_type_u32 {
            0 => CRConnectionType::Unknown,
            1 => CRConnectionType::Ethernet,
            2 => CRConnectionType::WiFi,
            3 => CRConnectionType::Vpn,
            4 => CRConnectionType::Bridge,
            5 => CRConnectionType::Bond,
            6 => CRConnectionType::Vlan,
            7 => CRConnectionType::Loopback,
            _ => return Err(fdo::Error::InvalidArgs(format!("Invalid connection type: {}", conn_type_u32))),
        };

        // Generate UUID
        let uuid = Uuid::new_v4().to_string();

        // Create connection info
        let mut conn_info = CRConnectionInfo::new(uuid.clone(), id.clone(), conn_type);

        // Set autoconnect if provided
        if let Some(autoconnect_val) = settings.get("Autoconnect") {
            if let Ok(ac) = autoconnect_val.downcast_ref::<bool>() {
                conn_info.autoconnect = ac;
            }
        }

        info!("CR Connection: Adding new connection {} ({})", id, uuid);

        // Add connection
        self.add_connection_internal(conn_info).await;

        // Connection will be persisted by integration layer

        Ok(uuid)
    }

    /// Modify an existing connection
    async fn modify_connection(&self, id: &str, settings: HashMap<String, Value<'_>>) -> fdo::Result<()> {
        let mut connections = self.connections.write().await;

        // Find connection by UUID or ID
        let conn_uuid = if connections.contains_key(id) {
            id.to_string()
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(uuid, _)| uuid.clone())
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        let conn = connections.get_mut(&conn_uuid)
            .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?;

        info!("CR Connection: Modifying connection {}", id);

        // Update fields from settings
        if let Some(new_id_val) = settings.get("ID") {
            if let Ok(new_id) = new_id_val.downcast_ref::<&str>() {
                conn.id = new_id.to_string();
            }
        }

        if let Some(autoconnect_val) = settings.get("Autoconnect") {
            if let Ok(ac) = autoconnect_val.downcast_ref::<bool>() {
                conn.autoconnect = ac;
            }
        }

        // Modification will be persisted by integration layer

        Ok(())
    }

    /// Delete a connection
    async fn delete_connection(&self, id: &str) -> fdo::Result<()> {
        let connections = self.connections.read().await;

        // Find connection by UUID or ID
        let conn_uuid = if connections.contains_key(id) {
            id.to_string()
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(uuid, _)| uuid.clone())
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        drop(connections); // Release read lock

        info!("CR Connection: Deleting connection {}", id);

        self.remove_connection_internal(&conn_uuid).await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        // Deletion will be handled by integration layer

        Ok(())
    }

    /// Activate a connection
    async fn activate_connection(&self, id: &str, device_path: &str) -> fdo::Result<()> {
        let mut connections = self.connections.write().await;

        // Find connection by UUID or ID
        let conn_uuid = if connections.contains_key(id) {
            id.to_string()
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(uuid, _)| uuid.clone())
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        let conn = connections.get_mut(&conn_uuid)
            .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?;

        info!("CR Connection: Activating connection {} on device {}", id, device_path);

        conn.state = CRConnectionState::Activating;
        conn.device = Some(device_path.to_string());

        // Activation will be handled by integration layer

        Ok(())
    }

    /// Deactivate a connection
    async fn deactivate_connection(&self, id: &str) -> fdo::Result<()> {
        let mut connections = self.connections.write().await;

        // Find connection by UUID or ID
        let conn_uuid = if connections.contains_key(id) {
            id.to_string()
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(uuid, _)| uuid.clone())
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        let conn = connections.get_mut(&conn_uuid)
            .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?;

        info!("CR Connection: Deactivating connection {}", id);

        conn.state = CRConnectionState::Deactivating;
        conn.device = None;

        // Deactivation will be handled by integration layer

        Ok(())
    }

    /// Reload all connection files from disk
    async fn reload_connections(&self) -> fdo::Result<()> {
        info!("CR Connection: Reloading all connections");
        // Reload will be handled by integration layer
        Ok(())
    }

    /// Load a specific connection file
    async fn load_connection_file(&self, filename: &str) -> fdo::Result<String> {
        info!("CR Connection: Loading connection from file {}", filename);
        // File loading will be handled by integration layer
        // For now, return a placeholder UUID
        Ok(Uuid::new_v4().to_string())
    }

    /// Import a connection from external format
    async fn import_connection(&self, conn_type: &str, file: &str) -> fdo::Result<String> {
        info!("CR Connection: Importing {} connection from {}", conn_type, file);
        // Import will be handled by integration layer
        // For now, return a placeholder UUID
        Ok(Uuid::new_v4().to_string())
    }

    /// Export a connection to external format
    async fn export_connection(&self, id: &str) -> fdo::Result<String> {
        let connections = self.connections.read().await;

        // Find connection by UUID or ID
        let _conn = if let Some(conn) = connections.get(id) {
            conn
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(_, conn)| conn)
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        info!("CR Connection: Exporting connection {}", id);

        // Export will be handled by integration layer
        // For now, return placeholder export data
        Ok(format!("# Exported connection: {}\n", id))
    }

    /// Clone a connection with a new name
    async fn clone_connection(&self, id: &str, new_name: &str) -> fdo::Result<String> {
        let connections = self.connections.read().await;

        // Find original connection
        let orig_conn = if let Some(conn) = connections.get(id) {
            conn.clone()
        } else {
            connections.iter()
                .find(|(_, conn)| conn.id == id)
                .map(|(_, conn)| conn.clone())
                .ok_or_else(|| fdo::Error::Failed(format!("Connection {} not found", id)))?
        };

        drop(connections); // Release read lock

        info!("CR Connection: Cloning connection {} to {}", id, new_name);

        // Create new connection with new UUID and name
        let new_uuid = Uuid::new_v4().to_string();
        let mut new_conn = CRConnectionInfo::new(new_uuid.clone(), new_name.to_string(), orig_conn.conn_type);
        new_conn.autoconnect = orig_conn.autoconnect;

        self.add_connection_internal(new_conn).await;

        Ok(new_uuid)
    }

    /// Get active connections
    async fn get_active_connections(&self) -> Vec<HashMap<String, Value<'static>>> {
        let connections = self.connections.read().await;
        let mut result = Vec::new();

        for (_, conn) in connections.iter() {
            if matches!(conn.state, CRConnectionState::Activated | CRConnectionState::Activating) {
                let mut conn_info = HashMap::new();
                conn_info.insert("UUID".to_string(), Value::new(conn.uuid.clone()));
                conn_info.insert("ID".to_string(), Value::new(conn.id.clone()));
                conn_info.insert("Type".to_string(), Value::new(conn.conn_type as u32));
                conn_info.insert("State".to_string(), Value::new(conn.state as u32));

                if let Some(ref device) = conn.device {
                    conn_info.insert("Device".to_string(), Value::new(device.clone()));
                }

                result.push(conn_info);
            }
        }

        debug!("CR Connection: Returning {} active connections", result.len());
        result
    }

    // ============ D-Bus Signals ============

    /// ConnectionAdded signal - emitted when a connection is added
    #[zbus(signal)]
    async fn connection_added(signal_emitter: &SignalEmitter<'_>, uuid: &str, id: &str) -> zbus::Result<()>;

    /// ConnectionRemoved signal - emitted when a connection is removed
    #[zbus(signal)]
    async fn connection_removed(signal_emitter: &SignalEmitter<'_>, uuid: &str) -> zbus::Result<()>;

    /// ConnectionUpdated signal - emitted when a connection is modified
    #[zbus(signal)]
    async fn connection_updated(signal_emitter: &SignalEmitter<'_>, uuid: &str) -> zbus::Result<()>;

    /// ConnectionActivated signal - emitted when a connection is activated
    #[zbus(signal)]
    async fn connection_activated(signal_emitter: &SignalEmitter<'_>, uuid: &str, device_path: &str) -> zbus::Result<()>;

    /// ConnectionDeactivated signal - emitted when a connection is deactivated
    #[zbus(signal)]
    async fn connection_deactivated(signal_emitter: &SignalEmitter<'_>, uuid: &str) -> zbus::Result<()>;
}

impl CRConnection {
    /// Helper to convert connection to HashMap
    fn connection_to_hashmap(&self, conn: &CRConnectionInfo) -> HashMap<String, Value<'static>> {
        let mut info = HashMap::new();
        info.insert("UUID".to_string(), Value::new(conn.uuid.clone()));
        info.insert("ID".to_string(), Value::new(conn.id.clone()));
        info.insert("Type".to_string(), Value::new(conn.conn_type as u32));
        info.insert("State".to_string(), Value::new(conn.state as u32));
        info.insert("Autoconnect".to_string(), Value::new(conn.autoconnect));
        info.insert("Path".to_string(), Value::new(conn.path.clone()));

        if let Some(ref device) = conn.device {
            info.insert("Device".to_string(), Value::new(device.clone()));
        }

        info
    }
}

impl Default for CRConnection {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper module for emitting connection signals
pub mod signals {
    use super::*;

    /// Emit ConnectionAdded signal
    pub async fn emit_connection_added(
        conn: &Connection,
        uuid: &str,
        id: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRConnection>(CR_CONNECTION_PATH)
            .await
        {
            CRConnection::connection_added(iface_ref.signal_emitter(), uuid, id)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ConnectionAdded: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ConnectionRemoved signal
    pub async fn emit_connection_removed(
        conn: &Connection,
        uuid: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRConnection>(CR_CONNECTION_PATH)
            .await
        {
            CRConnection::connection_removed(iface_ref.signal_emitter(), uuid)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ConnectionRemoved: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ConnectionUpdated signal
    pub async fn emit_connection_updated(
        conn: &Connection,
        uuid: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRConnection>(CR_CONNECTION_PATH)
            .await
        {
            CRConnection::connection_updated(iface_ref.signal_emitter(), uuid)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ConnectionUpdated: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ConnectionActivated signal
    pub async fn emit_connection_activated(
        conn: &Connection,
        uuid: &str,
        device_path: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRConnection>(CR_CONNECTION_PATH)
            .await
        {
            CRConnection::connection_activated(iface_ref.signal_emitter(), uuid, device_path)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ConnectionActivated: {}", e)))?;
        }
        Ok(())
    }

    /// Emit ConnectionDeactivated signal
    pub async fn emit_connection_deactivated(
        conn: &Connection,
        uuid: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRConnection>(CR_CONNECTION_PATH)
            .await
        {
            CRConnection::connection_deactivated(iface_ref.signal_emitter(), uuid)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit ConnectionDeactivated: {}", e)))?;
        }
        Ok(())
    }
}
