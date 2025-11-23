# D-Bus Interface Audit

## Overview

This document audits the current D-Bus interfaces against the CLI commands in `nccli.rs` to identify missing methods needed for the daemon/CLI split.

## Existing D-Bus Interfaces

### 1. org.crrouter.NetworkControl (Main Interface)

**Path:** `/org/crrouter/NetworkControl`
**File:** `src/cr_dbus/network_control.rs`

**Methods:**
- ✅ `GetVersion()` → String
- ✅ `GetDevices()` → Vec<String> (device paths)
- ✅ `GetDeviceByInterface(iface: String)` → String (device path)
- ✅ `GetDeviceInfo(device_path: String)` → HashMap<String, Value>
- ✅ `ActivateDevice(device_path: String)`
- ✅ `DeactivateDevice(device_path: String)`
- ✅ `GetState()` → u32 (network state)
- ✅ `GetConnectivity()` → u32
- ✅ `CheckConnectivity()` → u32
- ✅ `GetNetworkingEnabled()` → bool
- ✅ `SetNetworkingEnabledMethod(enabled: bool)`
- ✅ `GetWirelessEnabled()` → bool
- ✅ `SetWirelessEnabledMethod(enabled: bool)`
- ✅ `Reload()`

**Signals:**
- ✅ StateChanged(state: u32)
- ✅ DeviceAdded(device_path: String)
- ✅ DeviceRemoved(device_path: String)
- ✅ DeviceStateChanged(device_path: String, state: u32)
- ✅ ConnectivityChanged(connectivity: u32)
- ✅ PropertiesChanged(properties: HashMap)

### 2. org.crrouter.NetworkControl.WiFi

**Path:** `/org/crrouter/NetworkControl/WiFi`
**File:** `src/cr_dbus/wifi.rs`

**Methods:**
- ✅ `GetEnabled()` → bool
- ✅ `SetEnabled(enabled: bool)`
- ✅ `Scan()`
- ✅ `GetAccessPoints()` → Vec<HashMap<String, Value>>
- ✅ `GetCurrentSSID()` → String
- ✅ `Connect(ssid: String, password: String, security: u32)`
- ✅ `Disconnect()`
- ✅ `StartAccessPoint(ssid: String, password: String, channel: u32)`
- ✅ `StopAccessPoint()`
- ✅ `IsScanning()` → bool

**Signals:**
- ✅ ScanCompleted()
- ✅ AccessPointAdded(ssid: String, bssid: String)
- ✅ AccessPointRemoved(ssid: String, bssid: String)
- ✅ Connected(ssid: String)
- ✅ Disconnected()

### 3. org.crrouter.NetworkControl.VPN

**Path:** `/org/crrouter/NetworkControl/VPN`
**File:** `src/cr_dbus/vpn.rs`

**Methods:**
- ✅ `GetConnections()` → Vec<String> (connection names)
- ✅ `GetConnectionInfo(name: String)` → HashMap<String, Value>
- ✅ `ConnectOpenvpn(name: String, config_file: String)`
- ✅ `ConnectWireguard(name: String, config_file: String)`
- ✅ `ConnectIpsec(name: String, remote: String, auth_method: String, credentials: HashMap)`
- ✅ `ConnectArti(name: String, config: HashMap)`
- ✅ `Disconnect(name: String)`
- ✅ `GetState(name: String)` → u32
- ✅ `GetStatistics(name: String)` → HashMap<String, Value>
- ✅ `DeleteConnection(name: String)`

**Signals:**
- ✅ ConnectionAdded(name: String, vpn_type: u32)
- ✅ ConnectionRemoved(name: String)
- ✅ StateChanged(name: String, state: u32)
- ✅ Connected(name: String, local_ip: String)
- ✅ Disconnected(name: String)
- ✅ Error(name: String, error_message: String)

### 4. org.crrouter.NetworkControl.Device

**Path:** `/org/crrouter/NetworkControl/Devices/{interface}`
**File:** `src/cr_dbus/device.rs`

**Methods:**
- ✅ `GetInterface()` → String
- ✅ `GetDeviceType()` → u32
- ✅ `GetState()` → u32
- ✅ `GetIpv4Address()` → String
- ✅ `GetIpv6Address()` → String
- ✅ `GetHwAddress()` → String
- ✅ `GetMTU()` → u32
- ✅ `GetAllProperties()` → HashMap<String, Value>
- ✅ `Activate()`
- ✅ `Deactivate()`
- ✅ `SetMTU(mtu: u32)`

**Signals:**
- ✅ StateChanged(new_state: u32, old_state: u32, reason: u32)
- ✅ IPConfigChanged()

## CLI Command Coverage Analysis

### ✅ FULLY COVERED

#### General Commands
- ✅ Status - uses `GetState()`, `GetConnectivity()`
- ⚠️ Hostname - **not via D-Bus** (uses hostnamectl directly, OK)
- ⚠️ Permissions - **not via D-Bus** (static response, OK)
- ⚠️ Logging - **MISSING** (should add D-Bus method for daemon logging control)

#### Networking Commands
- ✅ On/Off - uses `SetNetworkingEnabledMethod()`
- ✅ Connectivity - uses `GetConnectivity()`, `CheckConnectivity()`

#### Radio Commands (partial)
- ✅ WiFi - uses `GetWirelessEnabled()`, `SetWirelessEnabledMethod()`
- ❌ WWAN - **MISSING** (no WWAN D-Bus interface)
- ⚠️ All - can aggregate WiFi state

### ❌ MAJOR GAPS - Connection Management

**CLI Commands:** `connection show/up/down/add/modify/edit/delete/reload/load/import/export/clone`

**Status:** ❌ **COMPLETELY MISSING**

**Required D-Bus Interface:** `org.crrouter.NetworkControl.Connection`

**Missing Methods:**
- ❌ `ListConnections()` → Vec<ConnectionInfo>
- ❌ `GetConnection(id: String)` → ConnectionDetails
- ❌ `AddConnection(settings: HashMap)`
- ❌ `ModifyConnection(id: String, settings: HashMap)`
- ❌ `DeleteConnection(id: String)`
- ❌ `ActivateConnection(id: String, device_path: String)`
- ❌ `DeactivateConnection(id: String)`
- ❌ `ReloadConnections()`
- ❌ `LoadConnectionFile(filename: String)`
- ❌ `ImportConnection(type: String, file: String)`
- ❌ `ExportConnection(id: String)` → String
- ❌ `CloneConnection(id: String, new_name: String)`

**Missing Signals:**
- ❌ `ConnectionAdded(id: String)`
- ❌ `ConnectionRemoved(id: String)`
- ❌ `ConnectionUpdated(id: String)`
- ❌ `ConnectionActivated(id: String, device_path: String)`
- ❌ `ConnectionDeactivated(id: String)`

### ⚠️ PARTIAL COVERAGE - Device Commands

**Covered:**
- ✅ Status/Show - uses `GetDevices()`, `GetDeviceInfo()`, `GetAllProperties()`
- ✅ Connect/Disconnect - uses `Activate()`, `Deactivate()`

**Missing:**
- ❌ Set (autoconnect, managed) - need `SetDeviceProperties()`
- ❌ Reapply - need `ReapplyConnection()`
- ❌ Modify - need `ModifyActiveConnection()`
- ❌ Delete - need `DeleteDevice()`
- ❌ LLDP - need `GetLLDPNeighbors()`

**Required Additions to org.crrouter.NetworkControl.Device:**
- ❌ `SetAutoconnect(enabled: bool)`
- ❌ `SetManaged(managed: bool)`
- ❌ `ReapplyConnection()`
- ❌ `ModifyActiveConnection(settings: HashMap)`
- ❌ `Delete()` (for software devices)
- ❌ `GetLLDPNeighbors()` → Vec<LLDPInfo>

### ✅ GOOD COVERAGE - WiFi Device Commands

**Covered:**
- ✅ List - uses `GetAccessPoints()`
- ✅ Connect - uses `Connect()`
- ✅ Hotspot - uses `StartAccessPoint()`
- ✅ Radio - uses `SetEnabled()`

**Minor Additions Needed:**
- ⚠️ Rescan parameter - current `Scan()` doesn't have force parameter
- ⚠️ Hidden network support - `Connect()` doesn't have hidden flag
- ⚠️ Private connection flag

**Suggested Enhancements:**
- Add `Connect` parameters: `hidden: bool`, `private: bool`
- Add `Scan` parameters: `force: bool`

### ⚠️ GOOD COVERAGE - VPN Commands

**Covered:**
- ✅ List - uses `GetConnections()`
- ✅ Show - uses `GetConnectionInfo()`
- ✅ Connect - uses `Connect*()` methods
- ✅ Disconnect - uses `Disconnect()`
- ✅ Delete - uses `DeleteConnection()`
- ✅ Status - uses `GetState()`
- ✅ Stats - uses `GetStatistics()`

**Missing:**
- ❌ Import - need `ImportConfig(type: String, file: String, name: String)`
- ❌ Export - need `ExportConfig(name: String)` → String
- ❌ Create from TOML - need `CreateConnection(config: String)`
- ❌ Backends - need `ListBackends()` → Vec<VpnBackend>

**Required Additions to org.crrouter.NetworkControl.VPN:**
- ❌ `ImportConfig(vpn_type: String, config_file: String, name: String)`
- ❌ `ExportConfig(name: String)` → String
- ❌ `CreateFromConfig(config_toml: String)` → String (connection name)
- ❌ `ListBackends()` → Vec<String> (wireguard, openvpn, ipsec, arti)

### ⚠️ PARTIAL COVERAGE - AP Commands

**Covered:**
- ✅ Start - uses `StartAccessPoint()`
- ✅ Stop - uses `StopAccessPoint()`

**Missing:**
- ❌ Status - need `GetAccessPointStatus()`
- ❌ Restart - can be Stop + Start, but dedicated method better

**Required Additions to org.crrouter.NetworkControl.WiFi:**
- ❌ `GetAccessPointStatus()` → HashMap<String, Value> (ssid, channel, clients, etc.)
- ❌ `RestartAccessPoint()`
- ❌ `GetAccessPointClients()` → Vec<ClientInfo>

### ❌ COMPLETELY MISSING - DHCP Commands

**CLI Commands:** `dhcp start/stop/status/leases`

**Status:** ❌ **NO D-BUS INTERFACE**

**Required D-Bus Interface:** `org.crrouter.NetworkControl.DHCP`

**Missing Methods:**
- ❌ `StartServer(interface: String, range_start: String, range_end: String, gateway: String, dns_servers: Vec<String>)`
- ❌ `StopServer()`
- ❌ `GetStatus()` → HashMap<String, Value>
- ❌ `GetLeases()` → Vec<LeaseInfo>
- ❌ `IsRunning()` → bool

**Missing Signals:**
- ❌ `ServerStarted()`
- ❌ `ServerStopped()`
- ❌ `LeaseAssigned(mac: String, ip: String, hostname: String)`
- ❌ `LeaseExpired(mac: String, ip: String)`

### ❌ COMPLETELY MISSING - DNS Commands

**CLI Commands:** `dns start/stop/status/flush`

**Status:** ❌ **NO D-BUS INTERFACE**

**Required D-Bus Interface:** `org.crrouter.NetworkControl.DNS`

**Missing Methods:**
- ❌ `StartServer(forwarders: Vec<String>)`
- ❌ `StopServer()`
- ❌ `GetStatus()` → HashMap<String, Value>
- ❌ `FlushCache()`
- ❌ `IsRunning()` → bool
- ❌ `GetCacheStats()` → HashMap<String, Value>

**Missing Signals:**
- ❌ `ServerStarted()`
- ❌ `ServerStopped()`
- ❌ `CacheFlushed()`

### ❌ COMPLETELY MISSING - Route Commands

**CLI Commands:** `route show/add-default/del-default`

**Status:** ❌ **NO D-BUS INTERFACE**

**Required D-Bus Interface:** `org.crrouter.NetworkControl.Route`

**Missing Methods:**
- ❌ `GetRoutingTable()` → Vec<RouteInfo>
- ❌ `AddDefaultGateway(gateway: String, interface: Option<String>)`
- ❌ `DeleteDefaultGateway()`
- ❌ `AddRoute(destination: String, gateway: String, metric: Option<u32>)`
- ❌ `DeleteRoute(destination: String)`

**Missing Signals:**
- ❌ `RouteAdded(destination: String, gateway: String)`
- ❌ `RouteRemoved(destination: String)`
- ❌ `DefaultGatewayChanged(gateway: String)`

### ⚠️ DEBUG COMMANDS - May Not Need D-Bus

**CLI Commands:** `debug ping/tcpdump`

**Status:** ⚠️ **CLI-ONLY ACCEPTABLE**

**Rationale:** Debug commands can directly call system tools (ping, tcpdump) without going through daemon. These are diagnostic tools, not network configuration.

**Decision:** Skip D-Bus for debug commands. CLI can execute directly.

### ⚠️ MONITOR COMMAND

**CLI Command:** `monitor`

**Status:** ⚠️ **SIGNALS AVAILABLE**

**Coverage:**
- ✅ Device changes - `DeviceAdded`, `DeviceRemoved`, `DeviceStateChanged`
- ✅ Network state - `StateChanged`, `ConnectivityChanged`
- ✅ WiFi - `Connected`, `Disconnected`, `ScanCompleted`
- ✅ VPN - `Connected`, `Disconnected`, `StateChanged`
- ❌ Connection changes - need Connection signals (currently missing)

## Summary of Missing D-Bus Methods

### Critical (Blocks daemon/CLI split):

1. **Connection Management** - Complete new interface needed
   - 12 methods for connection CRUD operations
   - 5 signals for connection events

2. **DHCP Server** - Complete new interface needed
   - 4 methods (start/stop/status/leases)
   - 4 signals

3. **DNS Server** - Complete new interface needed
   - 5 methods (start/stop/status/flush/stats)
   - 3 signals

4. **Routing** - Complete new interface needed
   - 5 methods (show/add/delete routes)
   - 3 signals

### Important (Needed for feature parity):

5. **Device Management Enhancements**
   - SetAutoconnect, SetManaged, Delete, GetLLDPNeighbors
   - ReapplyConnection, ModifyActiveConnection

6. **VPN Enhancements**
   - ImportConfig, ExportConfig, CreateFromConfig, ListBackends

7. **WiFi/AP Enhancements**
   - GetAccessPointStatus, RestartAccessPoint, GetAccessPointClients

8. **Network Control Enhancements**
   - SetLoggingLevel (for daemon logging control)
   - GetHostname, SetHostname (optional, currently uses hostnamectl)

### Optional (Nice to have):

9. **WWAN Radio Control**
   - GetWWANEnabled, SetWWANEnabled

## Implementation Priority

### Phase 1 - Essential (Week 1)
1. ✅ Audit complete (this document)
2. **Connection interface** - Required for basic network management
3. **Device enhancements** - SetAutoconnect, SetManaged

### Phase 2 - Infrastructure Services (Week 1-2)
4. **DHCP interface** - Access Point needs this
5. **DNS interface** - Complete networking stack
6. **Routing interface** - Network configuration

### Phase 3 - Enhancements (Week 2-3)
7. **VPN enhancements** - Import/Export
8. **WiFi/AP enhancements** - Status, clients
9. **Logging control**

### Phase 4 - Optional (Week 3-4)
10. **WWAN support**
11. Additional signals and properties

## D-Bus Type Definitions Needed

### New Type Structures:

```rust
// Connection types
pub struct ConnectionInfo {
    pub id: String,
    pub uuid: String,
    pub type_: String,
    pub device: Option<String>,
    pub state: u32,
}

// DHCP lease
pub struct LeaseInfo {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub expiry: u64,
}

// Route info
pub struct RouteInfo {
    pub destination: String,
    pub gateway: String,
    pub interface: String,
    pub metric: u32,
}

// LLDP neighbor
pub struct LLDPNeighborInfo {
    pub chassis_id: String,
    pub port_id: String,
    pub system_name: String,
    pub capabilities: Vec<String>,
}

// AP client
pub struct APClientInfo {
    pub mac: String,
    pub ip: Option<String>,
    pub signal_strength: i32,
    pub connected_time: u64,
}
```

## Testing Requirements

Each new D-Bus method must have:
1. Unit test in interface file
2. Integration test in `tests/dbus_comprehensive_test.rs`
3. Example in `examples/` directory
4. Documentation in D-Bus introspection XML

## Next Steps

1. ✅ Complete this audit
2. Create new D-Bus interface files:
   - `src/cr_dbus/connection.rs`
   - `src/cr_dbus/dhcp.rs`
   - `src/cr_dbus/dns.rs`
   - `src/cr_dbus/route.rs`
3. Enhance existing interfaces with missing methods
4. Update integration layer to wire everything up
5. Create D-Bus client library
6. Implement CLI using D-Bus client

## Files to Create/Modify

### New Files:
- `src/cr_dbus/connection.rs` - Connection management interface
- `src/cr_dbus/dhcp.rs` - DHCP server interface
- `src/cr_dbus/dns.rs` - DNS server interface
- `src/cr_dbus/route.rs` - Routing interface
- `src/dbus_client/mod.rs` - D-Bus client library
- `src/dbus_client/types.rs` - Client-side types
- `src/bin/netctld.rs` - Daemon binary
- `src/bin/netctl.rs` - CLI binary

### Files to Modify:
- `src/cr_dbus/device.rs` - Add missing methods
- `src/cr_dbus/vpn.rs` - Add Import/Export
- `src/cr_dbus/wifi.rs` - Add AP enhancements
- `src/cr_dbus/network_control.rs` - Add logging control
- `src/cr_dbus/mod.rs` - Register new interfaces
- `src/cr_dbus/integration.rs` - Wire up new interfaces

## Conclusion

**Total Missing:**
- 4 complete new D-Bus interfaces (Connection, DHCP, DNS, Route)
- ~35 new methods across all interfaces
- ~15 new signals
- Multiple type definitions

**Estimated Effort:**
- Phase 1 (Connection + Device): 2-3 days
- Phase 2 (DHCP, DNS, Route): 2-3 days
- Phase 3 (Enhancements): 2-3 days
- Testing & Integration: 2-3 days
- **Total: ~2 weeks** for complete D-Bus interface coverage

This represents significant work but is essential for a proper daemon/CLI separation.
