//! Device Management Module
//!
//! Provides comprehensive device discovery, monitoring, and control functionality.
//! This module serves as a unified interface for managing all types of network devices
//! including physical interfaces, WiFi adapters, virtual devices, and bridges.

use crate::error::{NetctlError, NetctlResult};
use crate::interface::{InterfaceController, InterfaceInfo};
use crate::wifi::WifiController;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::warn;

/// Network device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Physical ethernet adapter
    Ethernet,
    /// WiFi wireless adapter
    Wifi,
    /// Loopback interface
    Loopback,
    /// Bridge device
    Bridge,
    /// VLAN interface
    Vlan,
    /// TUN/TAP virtual interface
    TunTap,
    /// Virtual ethernet pair
    Veth,
    /// Bond/team interface
    Bond,
    /// VPN tunnel interface
    Vpn,
    /// Docker/container network
    Container,
    /// PPP interface
    Ppp,
    /// Unknown or other type
    Unknown,
}

/// Device state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    /// Device is up and operational
    Up,
    /// Device is down
    Down,
    /// Device exists but is not managed
    Unmanaged,
    /// Device is unavailable (e.g., rfkill for WiFi)
    Unavailable,
    /// Device is in error state
    Error,
    /// Unknown state
    Unknown,
}

/// Device capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Supports WiFi operations
    pub wifi: bool,
    /// Can operate as access point
    pub access_point: bool,
    /// Supports bridging
    pub bridge: bool,
    /// Supports VLAN tagging
    pub vlan: bool,
    /// Maximum transmission unit range
    pub mtu_range: Option<(u32, u32)>,
    /// Supported speeds (Mbps)
    pub speeds: Vec<u32>,
    /// Supports Wake-on-LAN
    pub wake_on_lan: bool,
}

/// Comprehensive device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device name (e.g., "eth0", "wlan0")
    pub name: String,
    /// Device index
    pub index: Option<u32>,
    /// Device type
    pub device_type: DeviceType,
    /// Current state
    pub state: DeviceState,
    /// MAC address
    pub mac_address: Option<String>,
    /// Current MTU
    pub mtu: Option<u32>,
    /// IP addresses (IPv4 and IPv6)
    pub addresses: Vec<String>,
    /// Device driver name
    pub driver: Option<String>,
    /// Device vendor
    pub vendor: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// PCI/USB bus information
    pub bus_info: Option<String>,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
    /// Interface flags (UP, BROADCAST, MULTICAST, etc.)
    pub flags: Vec<String>,
    /// Statistics (rx/tx bytes, packets, errors)
    pub stats: Option<DeviceStats>,
    /// Parent device (for virtual devices)
    pub parent: Option<String>,
    /// Associated devices (for bridges, bonds)
    pub children: Vec<String>,
}

/// Device statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    /// Bytes received
    pub rx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Receive drops
    pub rx_dropped: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Transmit drops
    pub tx_dropped: u64,
}

/// Device configuration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Set device state (up/down)
    pub state: Option<DeviceState>,
    /// Set MTU
    pub mtu: Option<u32>,
    /// Set MAC address
    pub mac_address: Option<String>,
    /// Add IP addresses
    pub add_addresses: Vec<String>,
    /// Remove IP addresses
    pub remove_addresses: Vec<String>,
}

/// Device controller for managing all network devices
pub struct DeviceController {
    interface_ctrl: InterfaceController,
    wifi_ctrl: WifiController,
    #[allow(dead_code)]
    device_cache: tokio::sync::RwLock<HashMap<String, Device>>,
}

impl DeviceController {
    /// Create a new device controller
    pub fn new() -> Self {
        Self {
            interface_ctrl: InterfaceController::new(),
            wifi_ctrl: WifiController::new(),
            device_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// List all devices
    pub async fn list(&self) -> NetctlResult<Vec<String>> {
        self.interface_ctrl.list().await
    }

    /// Get comprehensive information about a specific device
    pub async fn get_device(&self, name: &str) -> NetctlResult<Device> {
        // First get interface info
        let iface_info = self.interface_ctrl.get_info(name).await?;

        // Detect device type
        let device_type = self.detect_device_type(name, &iface_info).await;

        // Get driver information
        let (driver, vendor, model, bus_info) = self.get_device_metadata(name).await;

        // Get capabilities based on device type
        let capabilities = self.detect_capabilities(name, &device_type).await;

        // Determine state
        let state = self.determine_device_state(&iface_info);

        // Get parent and children for virtual devices
        let (parent, children) = self.get_device_hierarchy(name).await;

        // Extract statistics
        let stats = iface_info.stats.as_ref().map(|s| DeviceStats {
            rx_bytes: s.rx_bytes,
            rx_packets: s.rx_packets,
            rx_errors: s.rx_errors,
            rx_dropped: s.rx_dropped,
            tx_bytes: s.tx_bytes,
            tx_packets: s.tx_packets,
            tx_errors: s.tx_errors,
            tx_dropped: s.tx_dropped,
        });

        let device = Device {
            name: name.to_string(),
            index: iface_info.index,
            device_type,
            state,
            mac_address: iface_info.mac_address,
            mtu: iface_info.mtu,
            addresses: iface_info
                .addresses
                .iter()
                .map(|a| a.address.clone())
                .collect(),
            driver,
            vendor,
            model,
            bus_info,
            capabilities,
            flags: iface_info.flags,
            stats,
            parent,
            children,
        };

        Ok(device)
    }

    /// List all devices with full information
    pub async fn list_devices(&self) -> NetctlResult<Vec<Device>> {
        let names = self.list().await?;
        let mut devices = Vec::new();

        for name in names {
            match self.get_device(&name).await {
                Ok(device) => devices.push(device),
                Err(e) => {
                    warn!("Failed to get device info for {}: {}", name, e);
                }
            }
        }

        Ok(devices)
    }

    /// Configure a device
    pub async fn configure_device(
        &self,
        name: &str,
        config: &DeviceConfig,
    ) -> NetctlResult<()> {
        // Apply state changes
        if let Some(state) = config.state {
            match state {
                DeviceState::Up => self.interface_ctrl.up(name).await?,
                DeviceState::Down => self.interface_ctrl.down(name).await?,
                _ => {
                    return Err(NetctlError::InvalidParameter(format!(
                        "Cannot set device to state: {:?}",
                        state
                    )))
                }
            }
        }

        // Apply MTU changes
        if let Some(mtu) = config.mtu {
            self.interface_ctrl.set_mtu(name, mtu).await?;
        }

        // Apply MAC address changes
        if let Some(ref mac) = config.mac_address {
            self.interface_ctrl.set_mac(name, mac).await?;
        }

        // Add IP addresses
        for addr in &config.add_addresses {
            // Parse address to extract IP and prefix
            let parts: Vec<&str> = addr.split('/').collect();
            if parts.len() != 2 {
                return Err(NetctlError::InvalidParameter(format!(
                    "Invalid address format: {}. Expected IP/PREFIX",
                    addr
                )));
            }
            let ip = parts[0];
            let prefix = parts[1].parse::<u8>().map_err(|_| {
                NetctlError::InvalidParameter(format!("Invalid prefix: {}", parts[1]))
            })?;

            self.interface_ctrl.add_ip(name, ip, prefix).await?;
        }

        // Remove IP addresses
        for addr in &config.remove_addresses {
            let parts: Vec<&str> = addr.split('/').collect();
            if parts.len() != 2 {
                return Err(NetctlError::InvalidParameter(format!(
                    "Invalid address format: {}. Expected IP/PREFIX",
                    addr
                )));
            }
            let ip = parts[0];
            let prefix = parts[1].parse::<u8>().map_err(|_| {
                NetctlError::InvalidParameter(format!("Invalid prefix: {}", parts[1]))
            })?;

            self.interface_ctrl.del_ip(name, ip, prefix).await?;
        }

        Ok(())
    }

    /// Delete a virtual device
    pub async fn delete_device(&self, name: &str) -> NetctlResult<()> {
        // Verify it's a virtual device
        let device = self.get_device(name).await?;
        match device.device_type {
            DeviceType::Veth | DeviceType::Bridge | DeviceType::Vlan | DeviceType::TunTap
            | DeviceType::Bond => {
                // Delete using ip link
                self.interface_ctrl.delete(name).await
            }
            _ => Err(NetctlError::InvalidParameter(format!(
                "Cannot delete physical device: {}",
                name
            ))),
        }
    }

    /// Get devices by type
    pub async fn get_devices_by_type(&self, device_type: DeviceType) -> NetctlResult<Vec<Device>> {
        let all_devices = self.list_devices().await?;
        Ok(all_devices
            .into_iter()
            .filter(|d| d.device_type == device_type)
            .collect())
    }

    /// Detect device type based on name and interface info
    async fn detect_device_type(&self, name: &str, _info: &InterfaceInfo) -> DeviceType {
        // Check loopback
        if name == "lo" {
            return DeviceType::Loopback;
        }

        // Check for common naming patterns
        if name.starts_with("wlan") || name.starts_with("wlp") {
            return DeviceType::Wifi;
        }

        if name.starts_with("eth") || name.starts_with("en") || name.starts_with("eno") {
            // Could be ethernet, verify with driver info
            if let Ok(driver_path) = fs::read_link(format!("/sys/class/net/{}/device/driver", name)).await {
                if let Some(driver_name) = driver_path.file_name() {
                    let driver_str = driver_name.to_string_lossy();
                    // Check for WiFi drivers
                    if driver_str.contains("wifi")
                        || driver_str.contains("wl")
                        || driver_str.contains("ath")
                        || driver_str.contains("iwl")
                    {
                        return DeviceType::Wifi;
                    }
                }
            }
            return DeviceType::Ethernet;
        }

        if name.starts_with("br-") || name.starts_with("bridge") {
            return DeviceType::Bridge;
        }

        if name.starts_with("vlan") {
            return DeviceType::Vlan;
        }

        if name.starts_with("tun") || name.starts_with("tap") {
            return DeviceType::TunTap;
        }

        if name.starts_with("veth") {
            return DeviceType::Veth;
        }

        if name.starts_with("bond") {
            return DeviceType::Bond;
        }

        if name.starts_with("docker") || name.starts_with("veth") && name.len() > 10 {
            return DeviceType::Container;
        }

        if name.starts_with("ppp") {
            return DeviceType::Ppp;
        }

        if name.starts_with("wg") {
            return DeviceType::Vpn;
        }

        // Check if it's a wireless device by checking if wireless directory exists
        if Path::new(&format!("/sys/class/net/{}/wireless", name))
            .exists()
        {
            return DeviceType::Wifi;
        }

        DeviceType::Unknown
    }

    /// Get device metadata (driver, vendor, model)
    async fn get_device_metadata(
        &self,
        name: &str,
    ) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        let mut driver = None;
        let mut vendor = None;
        let mut model = None;
        let mut bus_info = None;

        // Get driver name
        if let Ok(driver_path) = fs::read_link(format!("/sys/class/net/{}/device/driver", name)).await {
            if let Some(driver_name) = driver_path.file_name() {
                driver = Some(driver_name.to_string_lossy().to_string());
            }
        }

        // Get vendor ID
        if let Ok(vendor_id) = fs::read_to_string(format!("/sys/class/net/{}/device/vendor", name)).await {
            vendor = Some(vendor_id.trim().to_string());
        }

        // Get device ID
        if let Ok(device_id) = fs::read_to_string(format!("/sys/class/net/{}/device/device", name)).await {
            model = Some(device_id.trim().to_string());
        }

        // Get bus info (PCI address)
        if let Ok(uevent) = fs::read_to_string(format!("/sys/class/net/{}/device/uevent", name)).await {
            for line in uevent.lines() {
                if line.starts_with("PCI_SLOT_NAME=") {
                    bus_info = Some(line.trim_start_matches("PCI_SLOT_NAME=").to_string());
                    break;
                }
            }
        }

        (driver, vendor, model, bus_info)
    }

    /// Detect device capabilities
    async fn detect_capabilities(&self, name: &str, device_type: &DeviceType) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();

        match device_type {
            DeviceType::Wifi => {
                caps.wifi = true;
                // Check if AP mode is supported
                if let Ok(_phy) = self.wifi_ctrl.get_phy(name).await {
                    // Could check for AP mode support here
                    caps.access_point = true;
                }
            }
            DeviceType::Ethernet => {
                caps.vlan = true;
                caps.bridge = true;
                // Check ethtool for speeds
                if let Ok(features) = fs::read_to_string(format!("/sys/class/net/{}/speed", name)).await {
                    if let Ok(speed) = features.trim().parse::<u32>() {
                        caps.speeds.push(speed);
                    }
                }
            }
            DeviceType::Bridge => {
                caps.bridge = true;
            }
            _ => {}
        }

        caps
    }

    /// Determine device state from interface info
    fn determine_device_state(&self, info: &InterfaceInfo) -> DeviceState {
        if let Some(ref state_str) = info.state {
            match state_str.to_uppercase().as_str() {
                "UP" => DeviceState::Up,
                "DOWN" => DeviceState::Down,
                _ => DeviceState::Unknown,
            }
        } else if info.flags.contains(&"UP".to_string()) {
            DeviceState::Up
        } else {
            DeviceState::Down
        }
    }

    /// Get device parent and children relationships
    async fn get_device_hierarchy(&self, name: &str) -> (Option<String>, Vec<String>) {
        let mut parent = None;
        let mut children = Vec::new();

        // Check for parent device (for VLAN, veth, etc.)
        // VLAN devices have a link to parent
        if let Ok(link) = fs::read_to_string(format!("/sys/class/net/{}/lower_*", name)).await {
            parent = Some(link.trim().to_string());
        }

        // Check for bridge members
        if let Ok(mut entries) = fs::read_dir(format!("/sys/class/net/{}/brif", name)).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(name) = entry.file_name().into_string() {
                    children.push(name);
                }
            }
        }

        (parent, children)
    }

    /// Monitor device events (to be integrated with NetworkMonitor)
    pub async fn monitor_devices(&self) -> NetctlResult<()> {
        // This would integrate with the existing NetworkMonitor
        // For now, this is a placeholder
        Ok(())
    }
}

impl Default for DeviceController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_controller_creation() {
        let controller = DeviceController::new();
        assert!(controller.list().await.is_ok());
    }

    #[tokio::test]
    async fn test_device_type_detection() {
        let controller = DeviceController::new();

        // Test loopback detection
        let iface_info = InterfaceInfo {
            name: "lo".to_string(),
            index: Some(1),
            mac_address: None,
            mtu: Some(65536),
            state: Some("UP".to_string()),
            flags: vec!["UP".to_string(), "LOOPBACK".to_string()],
            addresses: vec![],
            stats: None,
        };

        let device_type = controller.detect_device_type("lo", &iface_info).await;
        assert_eq!(device_type, DeviceType::Loopback);
    }
}
