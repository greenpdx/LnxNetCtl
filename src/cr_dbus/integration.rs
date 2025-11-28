//! CR D-Bus integration with netctl
//!
//! This module integrates the CR D-Bus interfaces with the netctl application,
//! allowing control of network operations through D-Bus.

use super::network_control::CRNetworkControl;
use super::wifi::CRWiFi;
use super::vpn::CRVPN;
use super::connection::CRConnection;
use super::dhcp::CRDhcp;
use super::dns::CRDns;
use super::routing::CRRouting;
use super::privilege::CRPrivilege;
use super::types::*;
use crate::error::{NetctlError, NetctlResult};
use crate::device::{DeviceController, Device};
use crate::wpa_supplicant::{WpaSupplicantController, WpaSecurityType};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use zbus::Connection;

/// CR D-Bus service manager
///
/// This struct manages all CR D-Bus interfaces and provides integration
/// with the netctl application.
pub struct CRDbusService {
    /// D-Bus connection
    connection: Arc<Connection>,
    /// Network control interface
    network_control: Arc<CRNetworkControl>,
    /// WiFi interface
    wifi: Arc<CRWiFi>,
    /// VPN interface
    vpn: Arc<CRVPN>,
    /// Connection management interface
    conn_mgmt: Arc<CRConnection>,
    /// DHCP server interface
    dhcp: Arc<CRDhcp>,
    /// DNS server interface
    dns: Arc<CRDns>,
    /// Routing interface
    routing: Arc<CRRouting>,
    /// Privilege token interface
    privilege: Arc<CRPrivilege>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// WPA Supplicant controller for WiFi operations
    wpa_supplicant: Arc<WpaSupplicantController>,
    /// Primary WiFi interface name (e.g., wlan0)
    wifi_interface: Arc<RwLock<Option<String>>>,
}

impl CRDbusService {
    /// Start the CR D-Bus service
    ///
    /// This initializes all D-Bus interfaces and registers them on the system bus.
    pub async fn start() -> NetctlResult<Arc<Self>> {
        info!("Starting CR D-Bus service");

        // Connect to system bus
        let connection = Connection::system().await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to connect to D-Bus: {}", e)))?;

        // Create interface instances
        let network_control = CRNetworkControl::new();
        let wifi = CRWiFi::new();
        let vpn = CRVPN::new();
        let conn_mgmt = CRConnection::new();
        let dhcp = CRDhcp::new();
        let dns = CRDns::new();
        let routing = CRRouting::new();
        let privilege = CRPrivilege::new();

        // Register network control interface
        connection
            .object_server()
            .at(CR_DBUS_PATH, network_control.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register NetworkControl: {}", e)))?;

        info!("Registered CR NetworkControl interface at {}", CR_DBUS_PATH);

        // Register WiFi interface
        connection
            .object_server()
            .at(CR_WIFI_PATH, wifi.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register WiFi: {}", e)))?;

        info!("Registered CR WiFi interface at {}", CR_WIFI_PATH);

        // Register VPN interface
        let vpn_path = "/org/crrouter/NetworkControl/VPN";
        connection
            .object_server()
            .at(vpn_path, vpn.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register VPN: {}", e)))?;

        info!("Registered CR VPN interface at {}", vpn_path);

        // Register Connection interface
        connection
            .object_server()
            .at(CR_CONNECTION_PATH, conn_mgmt.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register Connection: {}", e)))?;

        info!("Registered CR Connection interface at {}", CR_CONNECTION_PATH);

        // Register DHCP interface
        connection
            .object_server()
            .at(CR_DHCP_PATH, dhcp.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register DHCP: {}", e)))?;

        info!("Registered CR DHCP interface at {}", CR_DHCP_PATH);

        // Register DNS interface
        connection
            .object_server()
            .at(CR_DNS_PATH, dns.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register DNS: {}", e)))?;

        info!("Registered CR DNS interface at {}", CR_DNS_PATH);

        // Register Routing interface
        connection
            .object_server()
            .at(CR_ROUTING_PATH, routing.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register Routing: {}", e)))?;

        info!("Registered CR Routing interface at {}", CR_ROUTING_PATH);

        // Register Privilege interface
        connection
            .object_server()
            .at(CR_PRIVILEGE_PATH, privilege.clone())
            .await
            .map_err(|e| NetctlError::ServiceError(format!("Failed to register Privilege: {}", e)))?;

        info!("Registered CR Privilege interface at {}", CR_PRIVILEGE_PATH);

        // Store Arc references for later use
        let network_control = Arc::new(network_control);
        let wifi = Arc::new(wifi);
        let vpn = Arc::new(vpn);
        let conn_mgmt = Arc::new(conn_mgmt);
        let dhcp = Arc::new(dhcp);
        let dns = Arc::new(dns);
        let routing = Arc::new(routing);
        let privilege = Arc::new(privilege);

        // Request well-known name
        info!("Requesting D-Bus name: {}", CR_DBUS_SERVICE);
        match connection.request_name(CR_DBUS_SERVICE).await {
            Ok(_) => {
                info!("✓ Successfully registered D-Bus service: {}", CR_DBUS_SERVICE);
            }
            Err(e) => {
                error!("✗ Failed to request D-Bus name '{}': {}", CR_DBUS_SERVICE, e);
                // Don't fail - we can still operate without owning the name
            }
        }

        let service = Arc::new(Self {
            connection: Arc::new(connection),
            network_control,
            wifi,
            vpn,
            conn_mgmt,
            dhcp,
            dns,
            routing,
            privilege,
            running: Arc::new(RwLock::new(true)),
            wpa_supplicant: Arc::new(WpaSupplicantController::new()),
            wifi_interface: Arc::new(RwLock::new(None)),
        });

        info!("CR D-Bus service started successfully");
        Ok(service)
    }

    /// Stop the CR D-Bus service
    pub async fn stop(&self) -> NetctlResult<()> {
        info!("Stopping CR D-Bus service");
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Check if service is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get network control interface
    pub fn network_control(&self) -> Arc<CRNetworkControl> {
        self.network_control.clone()
    }

    /// Get WiFi interface
    pub fn wifi(&self) -> Arc<CRWiFi> {
        self.wifi.clone()
    }

    /// Get VPN interface
    pub fn vpn(&self) -> Arc<CRVPN> {
        self.vpn.clone()
    }

    /// Get Connection management interface
    pub fn connection_mgmt(&self) -> Arc<CRConnection> {
        self.conn_mgmt.clone()
    }

    /// Get DHCP server interface
    pub fn dhcp(&self) -> Arc<CRDhcp> {
        self.dhcp.clone()
    }

    /// Get DNS server interface
    pub fn dns(&self) -> Arc<CRDns> {
        self.dns.clone()
    }

    /// Get Routing interface
    pub fn routing(&self) -> Arc<CRRouting> {
        self.routing.clone()
    }

    /// Get Privilege interface
    pub fn privilege(&self) -> Arc<CRPrivilege> {
        self.privilege.clone()
    }

    /// Get D-Bus connection
    pub fn connection(&self) -> Arc<Connection> {
        self.connection.clone()
    }

    /// Discover and add network devices
    ///
    /// This scans for network devices and adds them to the D-Bus service.
    pub async fn discover_devices(&self) -> NetctlResult<()> {
        info!("Discovering network devices");

        // Create device controller to list devices
        let device_controller = DeviceController::new();

        match device_controller.list_devices().await {
            Ok(devices) => {
                for device in &devices {
                    let device_type = Self::map_device_type(&device);
                    let mut device_info = CRDeviceInfo::new(device.name.clone(), device_type);

                    // Set hardware address (MAC address) if available
                    if let Some(ref mac_addr) = device.mac_address {
                        device_info.hw_address = Some(mac_addr.clone());
                    }

                    // Set MTU if available
                    if let Some(mtu) = device.mtu {
                        device_info.mtu = mtu;
                    }

                    // Set IP addresses if available
                    if !device.addresses.is_empty() {
                        // Try to separate IPv4 and IPv6
                        for addr in &device.addresses {
                            if addr.contains(':') {
                                // IPv6
                                if device_info.ipv6_address.is_none() {
                                    device_info.ipv6_address = Some(addr.clone());
                                }
                            } else {
                                // IPv4
                                if device_info.ipv4_address.is_none() {
                                    device_info.ipv4_address = Some(addr.clone());
                                }
                            }
                        }
                    }

                    // Determine device state based on flags
                    device_info.state = if device.flags.contains(&"UP".to_string()) {
                        CRDeviceState::Activated
                    } else {
                        CRDeviceState::Disconnected
                    };

                    self.network_control.add_device(device_info.clone()).await;

                    // Emit signal
                    if let Err(e) = super::network_control::signals::emit_device_added(
                        &self.connection,
                        &device_info.path,
                    ).await {
                        warn!("Failed to emit DeviceAdded signal: {}", e);
                    }
                }

                info!("Discovered {} devices", devices.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to list devices: {}", e);
                Err(e)
            }
        }
    }

    /// Map device type from netctl Device to CRDeviceType
    fn map_device_type(device: &Device) -> CRDeviceType {
        // Determine device type based on interface name and properties
        let name = device.name.as_str();

        if name.starts_with("wl") || name.starts_with("wlan") {
            CRDeviceType::WiFi
        } else if name.starts_with("eth") || name.starts_with("en") {
            CRDeviceType::Ethernet
        } else if name == "lo" {
            CRDeviceType::Loopback
        } else if name.starts_with("br") {
            CRDeviceType::Bridge
        } else if name.starts_with("tun") || name.starts_with("tap") {
            CRDeviceType::TunTap
        } else if name.contains("vlan") {
            CRDeviceType::Vlan
        } else if name.starts_with("wg") {
            CRDeviceType::Vpn
        } else if name.starts_with("bt") || name.starts_with("bnep") {
            CRDeviceType::Bluetooth
        } else {
            CRDeviceType::Unknown
        }
    }

    /// Update device state
    pub async fn update_device_state(
        &self,
        interface: &str,
        state: CRDeviceState,
    ) -> NetctlResult<()> {
        let device_path = format!("{}/{}", CR_DEVICE_PATH_PREFIX, interface);
        self.network_control.update_device_state(&device_path, state).await?;

        // Emit signal
        if let Err(e) = super::network_control::signals::emit_device_state_changed(
            &self.connection,
            &device_path,
            state,
        ).await {
            warn!("Failed to emit DeviceStateChanged signal: {}", e);
        }

        Ok(())
    }

    /// Update network state
    pub async fn update_network_state(&self, state: CRNetworkState) -> NetctlResult<()> {
        self.network_control.set_network_state(state).await;

        // Emit signal
        if let Err(e) = super::network_control::signals::emit_state_changed(
            &self.connection,
            state,
        ).await {
            warn!("Failed to emit StateChanged signal: {}", e);
        }

        Ok(())
    }

    /// Update connectivity state
    pub async fn update_connectivity(&self, connectivity: CRConnectivity) -> NetctlResult<()> {
        self.network_control.set_connectivity(connectivity).await;

        // Emit signal
        if let Err(e) = super::network_control::signals::emit_connectivity_changed(
            &self.connection,
            connectivity,
        ).await {
            warn!("Failed to emit ConnectivityChanged signal: {}", e);
        }

        Ok(())
    }

    /// Update WiFi access points
    pub async fn update_wifi_access_points(&self, aps: Vec<CRAccessPointInfo>) -> NetctlResult<()> {
        self.wifi.update_access_points(aps).await;

        // Emit scan completed signal
        if let Err(e) = super::wifi::signals::emit_scan_completed(&self.connection).await {
            warn!("Failed to emit ScanCompleted signal: {}", e);
        }

        Ok(())
    }

    /// Add VPN connection
    pub async fn add_vpn_connection(&self, vpn_info: CRVpnInfo) -> NetctlResult<()> {
        let name = vpn_info.name.clone();
        let vpn_type = vpn_info.vpn_type;

        self.vpn.add_connection(vpn_info).await;

        // Emit signal
        if let Err(e) = super::vpn::signals::emit_connection_added(
            &self.connection,
            &name,
            vpn_type,
        ).await {
            warn!("Failed to emit ConnectionAdded signal: {}", e);
        }

        Ok(())
    }

    /// Update VPN state
    pub async fn update_vpn_state(&self, name: &str, state: CRVpnState) -> NetctlResult<()> {
        self.vpn.update_state(name, state).await?;

        // Emit signal
        if let Err(e) = super::vpn::signals::emit_state_changed(
            &self.connection,
            name,
            state,
        ).await {
            warn!("Failed to emit StateChanged signal: {}", e);
        }

        Ok(())
    }

    // ============ WiFi Operations ============

    /// Get or detect the primary WiFi interface
    pub async fn get_wifi_interface(&self) -> NetctlResult<String> {
        // Check if we already have a cached interface
        {
            let iface = self.wifi_interface.read().await;
            if let Some(ref name) = *iface {
                return Ok(name.clone());
            }
        }

        // Try to detect a WiFi interface
        let device_controller = DeviceController::new();
        let devices = device_controller.list_devices().await?;

        for device in devices {
            if device.name.starts_with("wl") || device.name.starts_with("wlan") {
                let mut iface = self.wifi_interface.write().await;
                *iface = Some(device.name.clone());
                info!("Detected WiFi interface: {}", device.name);
                return Ok(device.name);
            }
        }

        Err(NetctlError::NotFound("No WiFi interface found".to_string()))
    }

    /// Set the WiFi interface to use
    pub async fn set_wifi_interface(&self, interface: String) {
        let mut iface = self.wifi_interface.write().await;
        *iface = Some(interface.clone());
        info!("WiFi interface set to: {}", interface);
    }

    /// Scan for WiFi networks
    pub async fn wifi_scan(&self) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;

        info!("Starting WiFi scan on {}", interface);
        self.wifi.set_scanning(true).await;

        // Trigger scan
        self.wpa_supplicant.scan(&interface).await?;

        // Wait a bit for scan to complete
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Get scan results
        let scan_results = self.wpa_supplicant.scan_results(&interface).await?;

        // Convert to CRAccessPointInfo
        let mut access_points = Vec::new();
        for result in scan_results {
            let security = match result.security_type() {
                WpaSecurityType::None => CRWiFiSecurity::None,
                WpaSecurityType::Wep => CRWiFiSecurity::Wep,
                WpaSecurityType::WpaPsk => CRWiFiSecurity::Wpa,
                WpaSecurityType::Wpa2Psk => CRWiFiSecurity::Wpa2,
                WpaSecurityType::Wpa3Sae => CRWiFiSecurity::Wpa3,
                WpaSecurityType::WpaEap | WpaSecurityType::Wpa2Eap => CRWiFiSecurity::Enterprise,
            };

            access_points.push(CRAccessPointInfo {
                ssid: result.ssid.clone(),
                bssid: result.bssid.clone(),
                strength: result.signal_percent(),
                security,
                frequency: result.frequency,
                mode: CRWiFiMode::Infrastructure,
            });
        }

        info!("WiFi scan found {} access points", access_points.len());

        // Update the WiFi interface with results
        self.wifi.update_access_points(access_points).await;
        self.wifi.set_scanning(false).await;

        // Emit scan completed signal
        if let Err(e) = super::wifi::signals::emit_scan_completed(&self.connection).await {
            warn!("Failed to emit ScanCompleted signal: {}", e);
        }

        Ok(())
    }

    /// Connect to a WiFi network
    pub async fn wifi_connect(&self, ssid: &str, password: Option<&str>) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;

        info!("Connecting to WiFi network '{}' on {}", ssid, interface);

        // Connect using wpa_supplicant
        self.wpa_supplicant.connect(&interface, ssid, password).await?;

        // Update current SSID
        self.wifi.set_current_ssid(Some(ssid.to_string())).await;

        // Update device state
        if let Err(e) = self.update_device_state(&interface, CRDeviceState::Activated).await {
            warn!("Failed to update device state: {}", e);
        }

        // Emit connected signal
        if let Err(e) = super::wifi::signals::emit_connected(&self.connection, ssid).await {
            warn!("Failed to emit Connected signal: {}", e);
        }

        Ok(())
    }

    /// Disconnect from WiFi network
    pub async fn wifi_disconnect(&self) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;

        info!("Disconnecting WiFi on {}", interface);

        // Disconnect using wpa_supplicant
        self.wpa_supplicant.disconnect(&interface).await?;

        // Clear current SSID
        self.wifi.set_current_ssid(None).await;

        // Update device state
        if let Err(e) = self.update_device_state(&interface, CRDeviceState::Disconnected).await {
            warn!("Failed to update device state: {}", e);
        }

        // Emit disconnected signal
        if let Err(e) = super::wifi::signals::emit_disconnected(&self.connection).await {
            warn!("Failed to emit Disconnected signal: {}", e);
        }

        Ok(())
    }

    /// Get WiFi connection status
    pub async fn wifi_status(&self) -> NetctlResult<Option<String>> {
        let interface = match self.get_wifi_interface().await {
            Ok(iface) => iface,
            Err(_) => return Ok(None),
        };

        if !self.wpa_supplicant.is_running(&interface).await {
            return Ok(None);
        }

        match self.wpa_supplicant.status(&interface).await {
            Ok(status) => {
                if status.state == crate::wpa_supplicant::WpaState::Completed {
                    Ok(status.ssid)
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Get WiFi signal strength
    pub async fn wifi_signal_strength(&self) -> NetctlResult<i32> {
        let interface = self.get_wifi_interface().await?;
        self.wpa_supplicant.signal_poll(&interface).await
    }

    /// Check if wpa_supplicant is available
    pub async fn is_wpa_supplicant_available(&self) -> bool {
        self.wpa_supplicant.is_installed().await
    }

    /// Start wpa_supplicant on WiFi interface
    pub async fn start_wpa_supplicant(&self) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;
        self.wpa_supplicant.start(&interface).await
    }

    /// Stop wpa_supplicant on WiFi interface
    pub async fn stop_wpa_supplicant(&self) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;
        self.wpa_supplicant.stop(&interface).await
    }

    /// List saved WiFi networks
    pub async fn wifi_list_networks(&self) -> NetctlResult<Vec<(String, String, String)>> {
        let interface = self.get_wifi_interface().await?;
        self.wpa_supplicant.list_networks(&interface).await
    }

    /// Remove a saved WiFi network
    pub async fn wifi_remove_network(&self, network_id: &str) -> NetctlResult<()> {
        let interface = self.get_wifi_interface().await?;
        self.wpa_supplicant.remove_network(&interface, network_id).await
    }
}
