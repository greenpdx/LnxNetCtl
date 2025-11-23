//! CR Routing D-Bus interface
//!
//! D-Bus interface for routing table management

use super::types::*;
use crate::error::{NetctlError, NetctlResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};
use zbus::{Connection, fdo, interface};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

/// CR Routing D-Bus interface
#[derive(Clone)]
pub struct CRRouting {
    /// Routing table (destination -> route info)
    routes: Arc<RwLock<HashMap<String, CRRouteInfo>>>,
    /// Default gateway (IPv4)
    default_gateway: Arc<RwLock<Option<String>>>,
    /// Default gateway (IPv6)
    default_gateway6: Arc<RwLock<Option<String>>>,
}

impl CRRouting {
    /// Create a new CR Routing interface
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            default_gateway: Arc::new(RwLock::new(None)),
            default_gateway6: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a route internally
    pub async fn add_route_internal(&self, route: CRRouteInfo) {
        let mut routes = self.routes.write().await;
        let key = route.destination.clone();
        routes.insert(key, route);
    }

    /// Remove a route internally
    pub async fn remove_route_internal(&self, destination: &str) -> bool {
        let mut routes = self.routes.write().await;
        routes.remove(destination).is_some()
    }

    /// Set default gateway
    pub async fn set_default_gateway_internal(&self, gateway: Option<String>, ipv6: bool) {
        if ipv6 {
            let mut gw = self.default_gateway6.write().await;
            *gw = gateway;
        } else {
            let mut gw = self.default_gateway.write().await;
            *gw = gateway;
        }
    }
}

#[interface(name = "org.crrouter.NetworkControl.Routing")]
impl CRRouting {
    /// Add a new route
    async fn add_route(
        &self,
        destination: &str,
        gateway: &str,
        interface: &str,
        metric: u32,
    ) -> fdo::Result<()> {
        info!(
            "CR Routing: Adding route {} via {} dev {} metric {}",
            destination, gateway, interface, metric
        );

        // Validate parameters
        if destination.is_empty() {
            return Err(fdo::Error::InvalidArgs("Destination cannot be empty".to_string()));
        }

        let mut route = CRRouteInfo::new(destination.to_string());

        if !gateway.is_empty() {
            route.gateway = Some(gateway.to_string());
        }

        if !interface.is_empty() {
            route.interface = Some(interface.to_string());
        }

        route.metric = metric;

        self.add_route_internal(route).await;

        // Actual route addition will be handled by integration layer

        Ok(())
    }

    /// Remove a route
    async fn remove_route(&self, destination: &str) -> fdo::Result<()> {
        info!("CR Routing: Removing route {}", destination);

        if !self.remove_route_internal(destination).await {
            return Err(fdo::Error::Failed(format!("Route not found: {}", destination)));
        }

        // Actual route removal will be handled by integration layer

        Ok(())
    }

    /// Get all routes
    async fn get_routes(&self) -> Vec<HashMap<String, Value<'static>>> {
        let routes = self.routes.read().await;
        let mut result = Vec::new();

        for (_dest, route) in routes.iter() {
            let mut route_info = HashMap::new();
            route_info.insert("Destination".to_string(), Value::new(route.destination.clone()));

            if let Some(ref gw) = route.gateway {
                route_info.insert("Gateway".to_string(), Value::new(gw.clone()));
            }

            if let Some(ref iface) = route.interface {
                route_info.insert("Interface".to_string(), Value::new(iface.clone()));
            }

            route_info.insert("Metric".to_string(), Value::new(route.metric));

            let route_type_u32: u32 = route.route_type.into();
            route_info.insert("Type".to_string(), Value::new(route_type_u32));

            route_info.insert("Table".to_string(), Value::new(route.table));
            route_info.insert("Scope".to_string(), Value::new(route.scope));

            result.push(route_info);
        }

        debug!("CR Routing: Returning {} routes", result.len());
        result
    }

    /// Get route count
    async fn get_route_count(&self) -> u32 {
        let routes = self.routes.read().await;
        routes.len() as u32
    }

    /// Get a specific route
    async fn get_route(&self, destination: &str) -> fdo::Result<HashMap<String, Value<'static>>> {
        let routes = self.routes.read().await;

        if let Some(route) = routes.get(destination) {
            let mut route_info = HashMap::new();
            route_info.insert("Destination".to_string(), Value::new(route.destination.clone()));

            if let Some(ref gw) = route.gateway {
                route_info.insert("Gateway".to_string(), Value::new(gw.clone()));
            }

            if let Some(ref iface) = route.interface {
                route_info.insert("Interface".to_string(), Value::new(iface.clone()));
            }

            route_info.insert("Metric".to_string(), Value::new(route.metric));

            let route_type_u32: u32 = route.route_type.into();
            route_info.insert("Type".to_string(), Value::new(route_type_u32));

            route_info.insert("Table".to_string(), Value::new(route.table));
            route_info.insert("Scope".to_string(), Value::new(route.scope));

            Ok(route_info)
        } else {
            Err(fdo::Error::Failed(format!("Route not found: {}", destination)))
        }
    }

    /// Set default gateway
    async fn set_default_gateway(&self, gateway: &str, interface: &str) -> fdo::Result<()> {
        info!("CR Routing: Setting default gateway to {} dev {}", gateway, interface);

        if gateway.is_empty() {
            return Err(fdo::Error::InvalidArgs("Gateway cannot be empty".to_string()));
        }

        // Determine if IPv6 based on presence of colons
        let is_ipv6 = gateway.contains(':');

        // Create default route
        let mut route = CRRouteInfo::new("default".to_string());
        route.gateway = Some(gateway.to_string());

        if !interface.is_empty() {
            route.interface = Some(interface.to_string());
        }

        route.metric = 0; // Default gateway has lowest metric

        // Store in routes
        self.add_route_internal(route).await;

        // Also store in default gateway field
        self.set_default_gateway_internal(Some(gateway.to_string()), is_ipv6).await;

        // Actual default gateway setting will be handled by integration layer

        Ok(())
    }

    /// Get default gateway
    async fn get_default_gateway(&self) -> HashMap<String, Value<'static>> {
        let mut result = HashMap::new();

        if let Some(ref gw) = *self.default_gateway.read().await {
            result.insert("Gateway".to_string(), Value::new(gw.clone()));
            result.insert("IPv6".to_string(), Value::new(false));
        }

        if let Some(ref gw6) = *self.default_gateway6.read().await {
            result.insert("Gateway6".to_string(), Value::new(gw6.clone()));
            result.insert("IPv6".to_string(), Value::new(true));
        }

        debug!("CR Routing: Returning default gateway info");
        result
    }

    /// Clear default gateway
    async fn clear_default_gateway(&self, ipv6: bool) -> fdo::Result<()> {
        info!("CR Routing: Clearing default gateway (IPv6: {})", ipv6);

        // Clear from default gateway field
        self.set_default_gateway_internal(None, ipv6).await;

        // Remove default route from routing table
        self.remove_route_internal("default").await;

        // Actual default gateway clearing will be handled by integration layer

        Ok(())
    }

    /// Clear all routes (dangerous operation, use with caution)
    async fn clear_all_routes(&self) -> fdo::Result<()> {
        warn!("CR Routing: Clearing ALL routes - this may break connectivity!");

        let mut routes = self.routes.write().await;
        routes.clear();

        // Clear default gateways
        let mut gw = self.default_gateway.write().await;
        *gw = None;
        drop(gw);

        let mut gw6 = self.default_gateway6.write().await;
        *gw6 = None;

        // Actual route clearing will be handled by integration layer

        Ok(())
    }

    // ============ D-Bus Signals ============

    /// RouteAdded signal - emitted when a route is added
    #[zbus(signal)]
    async fn route_added(
        signal_emitter: &SignalEmitter<'_>,
        destination: &str,
        gateway: &str,
        interface: &str,
    ) -> zbus::Result<()>;

    /// RouteRemoved signal - emitted when a route is removed
    #[zbus(signal)]
    async fn route_removed(
        signal_emitter: &SignalEmitter<'_>,
        destination: &str,
    ) -> zbus::Result<()>;

    /// DefaultGatewayChanged signal - emitted when default gateway changes
    #[zbus(signal)]
    async fn default_gateway_changed(
        signal_emitter: &SignalEmitter<'_>,
        gateway: &str,
        ipv6: bool,
    ) -> zbus::Result<()>;
}

impl Default for CRRouting {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper module for emitting routing signals
pub mod signals {
    use super::*;

    /// Emit RouteAdded signal
    pub async fn emit_route_added(
        conn: &Connection,
        destination: &str,
        gateway: &str,
        interface: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRRouting>(CR_ROUTING_PATH)
            .await
        {
            CRRouting::route_added(iface_ref.signal_emitter(), destination, gateway, interface)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit RouteAdded: {}", e)))?;
        }
        Ok(())
    }

    /// Emit RouteRemoved signal
    pub async fn emit_route_removed(
        conn: &Connection,
        destination: &str,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRRouting>(CR_ROUTING_PATH)
            .await
        {
            CRRouting::route_removed(iface_ref.signal_emitter(), destination)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit RouteRemoved: {}", e)))?;
        }
        Ok(())
    }

    /// Emit DefaultGatewayChanged signal
    pub async fn emit_default_gateway_changed(
        conn: &Connection,
        gateway: &str,
        ipv6: bool,
    ) -> NetctlResult<()> {
        if let Ok(iface_ref) = conn
            .object_server()
            .interface::<_, CRRouting>(CR_ROUTING_PATH)
            .await
        {
            CRRouting::default_gateway_changed(iface_ref.signal_emitter(), gateway, ipv6)
                .await
                .map_err(|e| NetctlError::ServiceError(format!("Failed to emit DefaultGatewayChanged: {}", e)))?;
        }
        Ok(())
    }
}
