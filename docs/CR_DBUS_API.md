# CR D-Bus Interface API Documentation

## Overview

The CR D-Bus interface provides a NetworkManager-like D-Bus API for controlling network operations through the netctl application. It uses the CR/cr*** naming convention to distinguish it from the standard NetworkManager D-Bus interface.

**Service Name:** `org.crrouter.NetworkControl`

## Architecture

The CR D-Bus interface consists of four main components:

1. **Network Control Interface** - Main network control and device management
2. **WiFi Interface** - WiFi operations (scanning, connecting, AP mode)
3. **VPN Interface** - VPN connection management
4. **Device Interface** - Individual device control (per-device)

## D-Bus Service Structure

### Service Information

- **Service Name:** `org.crrouter.NetworkControl`
- **Main Object Path:** `/org/crrouter/NetworkControl`
- **WiFi Object Path:** `/org/crrouter/NetworkControl/WiFi`
- **VPN Object Path:** `/org/crrouter/NetworkControl/VPN`
- **Device Object Paths:** `/org/crrouter/NetworkControl/Devices/{interface_name}`

## Network Control Interface

**Interface Name:** `org.crrouter.NetworkControl`
**Object Path:** `/org/crrouter/NetworkControl`

### Methods

#### GetVersion() → String
Get the API version.

**Returns:** Version string (e.g., "0.1.0")

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetVersion
```

#### GetDevices() → Array of Object Paths
Get all network devices.

**Returns:** Array of device object paths

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetDevices
```

#### GetDeviceByInterface(String interface_name) → Object Path
Get a device by its interface name.

**Parameters:**
- `interface_name` - Network interface name (e.g., "eth0", "wlan0")

**Returns:** Device object path

**Errors:**
- Returns error if device not found

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetDeviceByInterface \
  string:"eth0"
```

#### GetDeviceInfo(String device_path) → Dictionary
Get device information as a dictionary.

**Parameters:**
- `device_path` - Device object path

**Returns:** Dictionary with keys:
- `Interface` - Interface name (String)
- `DeviceType` - Device type (UInt32)
- `State` - Device state (UInt32)
- `IPv4Address` - IPv4 address if available (String)
- `IPv6Address` - IPv6 address if available (String)
- `HwAddress` - Hardware (MAC) address if available (String)
- `Mtu` - MTU (UInt32)

#### ActivateDevice(String device_path)
Activate a device (bring it up).

**Parameters:**
- `device_path` - Device object path

#### DeactivateDevice(String device_path)
Deactivate a device (bring it down).

**Parameters:**
- `device_path` - Device object path

#### GetState() → UInt32
Get global network state.

**Returns:** Network state value (see CRNetworkState enum)

#### GetConnectivity() → UInt32
Get connectivity state.

**Returns:** Connectivity value (see CRConnectivity enum)

#### CheckConnectivity() → UInt32
Perform active connectivity check.

**Returns:** Current connectivity value

#### GetNetworkingEnabled() → Boolean
Get networking enabled state.

**Returns:** True if networking is enabled

#### SetNetworkingEnabled(Boolean enabled)
Set networking enabled state.

**Parameters:**
- `enabled` - Enable or disable networking

#### GetWirelessEnabled() → Boolean
Get wireless enabled state.

**Returns:** True if wireless is enabled

#### SetWirelessEnabled(Boolean enabled)
Set wireless enabled state.

**Parameters:**
- `enabled` - Enable or disable wireless

#### Reload()
Reload configuration.

### Signals

#### StateChanged(UInt32 state)
Emitted when global network state changes.

**Parameters:**
- `state` - New network state

#### DeviceAdded(String device_path)
Emitted when a device is added.

**Parameters:**
- `device_path` - Path of the added device

#### DeviceRemoved(String device_path)
Emitted when a device is removed.

**Parameters:**
- `device_path` - Path of the removed device

#### DeviceStateChanged(String device_path, UInt32 state)
Emitted when a device state changes.

**Parameters:**
- `device_path` - Device object path
- `state` - New device state

#### ConnectivityChanged(UInt32 connectivity)
Emitted when connectivity changes.

**Parameters:**
- `connectivity` - New connectivity value

#### PropertiesChanged(Dictionary properties)
Emitted when properties change.

**Parameters:**
- `properties` - Changed properties

## WiFi Interface

**Interface Name:** `org.crrouter.NetworkControl.WiFi`
**Object Path:** `/org/crrouter/NetworkControl/WiFi`

### Methods

#### GetEnabled() → Boolean
Get WiFi enabled state.

**Returns:** True if WiFi is enabled

#### SetEnabled(Boolean enabled)
Set WiFi enabled state.

**Parameters:**
- `enabled` - Enable or disable WiFi

#### Scan()
Start a WiFi scan.

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl/WiFi \
  org.crrouter.NetworkControl.WiFi.Scan
```

#### GetAccessPoints() → Array of Dictionaries
Get list of scanned access points.

**Returns:** Array of dictionaries, each containing:
- `SSID` - Network name (String)
- `BSSID` - MAC address (String)
- `Strength` - Signal strength 0-100 (Byte)
- `Security` - Security type (UInt32)
- `Frequency` - Frequency in MHz (UInt32)
- `Mode` - WiFi mode (UInt32)

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl/WiFi \
  org.crrouter.NetworkControl.WiFi.GetAccessPoints
```

#### GetCurrentSsid() → String
Get current connected SSID.

**Returns:** SSID string (empty if not connected)

#### Connect(String ssid, String password, UInt32 security)
Connect to a WiFi network.

**Parameters:**
- `ssid` - Network SSID
- `password` - Network password
- `security` - Security type (0=None, 1=WEP, 2=WPA, 3=WPA2, 4=WPA3, 5=Enterprise)

**Example:**
```bash
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl/WiFi \
  org.crrouter.NetworkControl.WiFi.Connect \
  string:"MyNetwork" string:"password123" uint32:3
```

#### Disconnect()
Disconnect from current WiFi network.

#### StartAccessPoint(String ssid, String password, UInt32 channel)
Start WiFi Access Point mode.

**Parameters:**
- `ssid` - AP SSID
- `password` - AP password
- `channel` - WiFi channel

#### StopAccessPoint()
Stop WiFi Access Point mode.

#### IsScanning() → Boolean
Check if scanning is in progress.

**Returns:** True if scanning

### Signals

#### ScanCompleted()
Emitted when a scan completes.

#### AccessPointAdded(String ssid, String bssid)
Emitted when a new AP is detected.

#### AccessPointRemoved(String ssid, String bssid)
Emitted when an AP is no longer visible.

#### Connected(String ssid)
Emitted when connected to a network.

#### Disconnected()
Emitted when disconnected from a network.

## VPN Interface

**Interface Name:** `org.crrouter.NetworkControl.VPN`
**Object Path:** `/org/crrouter/NetworkControl/VPN`

### Methods

#### GetConnections() → Array of Strings
Get list of VPN connection names.

**Returns:** Array of VPN connection names

#### GetConnectionInfo(String name) → Dictionary
Get VPN connection information.

**Parameters:**
- `name` - VPN connection name

**Returns:** Dictionary with keys:
- `Name` - Connection name (String)
- `Path` - Object path (String)
- `Type` - VPN type (UInt32)
- `State` - VPN state (UInt32)
- `LocalIP` - Local IP if connected (String)
- `RemoteAddress` - Remote server address (String)

#### ConnectOpenVPN(String name, String config_file)
Connect to OpenVPN.

**Parameters:**
- `name` - Connection name
- `config_file` - Path to OpenVPN config file

#### ConnectWireGuard(String name, String config_file)
Connect to WireGuard VPN.

**Parameters:**
- `name` - Connection name
- `config_file` - Path to WireGuard config file

#### ConnectIPsec(String name, String remote, String auth_method, Dictionary credentials)
Connect to IPsec VPN.

**Parameters:**
- `name` - Connection name
- `remote` - Remote server address
- `auth_method` - Authentication method
- `credentials` - Authentication credentials

#### ConnectArti(String name, Dictionary config)
Connect to Arti/Tor.

**Parameters:**
- `name` - Connection name
- `config` - Tor configuration

#### Disconnect(String name)
Disconnect from a VPN.

**Parameters:**
- `name` - Connection name

#### GetState(String name) → UInt32
Get VPN connection state.

**Parameters:**
- `name` - Connection name

**Returns:** VPN state value

#### GetStatistics(String name) → Dictionary
Get statistics for a VPN connection.

**Parameters:**
- `name` - Connection name

**Returns:** Dictionary with statistics

#### DeleteConnection(String name)
Delete a VPN connection configuration.

**Parameters:**
- `name` - Connection name

### Signals

#### ConnectionAdded(String name, UInt32 vpn_type)
Emitted when a VPN connection is added.

#### ConnectionRemoved(String name)
Emitted when a VPN connection is removed.

#### StateChanged(String name, UInt32 state)
Emitted when VPN state changes.

#### Connected(String name, String local_ip)
Emitted when VPN is connected.

#### Disconnected(String name)
Emitted when VPN is disconnected.

#### Error(String name, String error_message)
Emitted when an error occurs.

## Device Interface

**Interface Name:** `org.crrouter.NetworkControl.Device`
**Object Paths:** `/org/crrouter/NetworkControl/Devices/{interface_name}`

### Methods

#### GetInterface() → String
Get device interface name.

#### GetDeviceType() → UInt32
Get device type.

#### GetState() → UInt32
Get device state.

#### GetIpv4Address() → String
Get IPv4 address.

#### GetIpv6Address() → String
Get IPv6 address.

#### GetHwAddress() → String
Get hardware (MAC) address.

#### GetMtu() → UInt32
Get MTU.

#### GetAllProperties() → Dictionary
Get all device properties.

#### Activate()
Activate the device.

#### Deactivate()
Deactivate the device.

#### SetMtu(UInt32 mtu)
Set MTU.

### Signals

#### StateChanged(UInt32 new_state, UInt32 old_state, UInt32 reason)
Emitted when device state changes.

#### IPConfigChanged()
Emitted when IP configuration changes.

## Enumerations

### CRNetworkState

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | Network is unknown |
| 10 | Initializing | Network is initializing |
| 20 | Disconnected | Network is disconnected |
| 30 | Connecting | Network is connecting |
| 40 | ConnectedLocal | Connected locally |
| 50 | ConnectedSite | Connected to site |
| 60 | ConnectedGlobal | Fully connected (internet) |

### CRConnectivity

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | Connectivity unknown |
| 1 | None | No connectivity |
| 2 | Limited | Limited connectivity |
| 3 | Portal | Captive portal detected |
| 4 | Full | Full connectivity |

### CRDeviceType

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | Unknown device |
| 1 | Ethernet | Ethernet device |
| 2 | WiFi | WiFi device |
| 3 | Bluetooth | Bluetooth device |
| 4 | Bridge | Bridge device |
| 5 | Vlan | VLAN device |
| 6 | TunTap | TUN/TAP device |
| 7 | Vpn | VPN device |
| 8 | Loopback | Loopback device |

### CRDeviceState

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | State unknown |
| 10 | Unmanaged | Device unmanaged |
| 20 | Unavailable | Device unavailable |
| 30 | Disconnected | Device disconnected |
| 40 | Preparing | Preparing to connect |
| 50 | Configuring | Being configured |
| 60 | NeedAuth | Needs authentication |
| 70 | IpConfig | IP configuration in progress |
| 80 | IpCheck | IP connectivity check |
| 90 | Secondaries | Waiting for secondaries |
| 100 | Activated | Device activated |
| 110 | Deactivating | Device deactivating |
| 120 | Failed | Device failed |

### CRWiFiSecurity

| Value | Name | Description |
|-------|------|-------------|
| 0 | None | No security (open) |
| 1 | Wep | WEP security |
| 2 | Wpa | WPA security |
| 3 | Wpa2 | WPA2 security |
| 4 | Wpa3 | WPA3 security |
| 5 | Enterprise | Enterprise security |

### CRVpnType

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | Unknown VPN |
| 1 | OpenVpn | OpenVPN |
| 2 | WireGuard | WireGuard |
| 3 | IPsec | IPsec |
| 4 | Arti | Arti/Tor |

### CRVpnState

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unknown | VPN unknown |
| 1 | Disconnected | VPN disconnected |
| 2 | Connecting | VPN connecting |
| 3 | Connected | VPN connected |
| 4 | Disconnecting | VPN disconnecting |
| 5 | Failed | VPN failed |

## Usage Examples

### Python with dbus-python

```python
import dbus

# Connect to system bus
bus = dbus.SystemBus()

# Get network control interface
proxy = bus.get_object('org.crrouter.NetworkControl',
                       '/org/crrouter/NetworkControl')
network_control = dbus.Interface(proxy, 'org.crrouter.NetworkControl')

# List devices
devices = network_control.GetDevices()
print(f"Found {len(devices)} devices:")
for device_path in devices:
    info = network_control.GetDeviceInfo(device_path)
    print(f"  {info['Interface']}: {info.get('IPv4Address', 'No IP')}")

# WiFi operations
wifi_proxy = bus.get_object('org.crrouter.NetworkControl',
                            '/org/crrouter/NetworkControl/WiFi')
wifi = dbus.Interface(wifi_proxy, 'org.crrouter.NetworkControl.WiFi')

# Scan for networks
wifi.Scan()

# Get access points
access_points = wifi.GetAccessPoints()
print(f"\nFound {len(access_points)} WiFi networks:")
for ap in access_points:
    print(f"  {ap['SSID']}: {ap['Strength']}% ({ap['BSSID']})")

# Connect to a network
wifi.Connect("MyNetwork", "password123", 3)  # 3 = WPA2
```

### Monitoring Signals with Python

```python
import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

DBusGMainLoop(set_as_default=True)

bus = dbus.SystemBus()

def device_added_handler(device_path):
    print(f"Device added: {device_path}")

def connectivity_changed_handler(connectivity):
    connectivity_names = {0: "Unknown", 1: "None", 2: "Limited",
                         3: "Portal", 4: "Full"}
    print(f"Connectivity changed: {connectivity_names.get(connectivity, 'Unknown')}")

# Subscribe to signals
bus.add_signal_receiver(
    device_added_handler,
    dbus_interface='org.crrouter.NetworkControl',
    signal_name='DeviceAdded'
)

bus.add_signal_receiver(
    connectivity_changed_handler,
    dbus_interface='org.crrouter.NetworkControl',
    signal_name='ConnectivityChanged'
)

# Run event loop
loop = GLib.MainLoop()
loop.run()
```

### Command Line with dbus-send

```bash
# Monitor all CR D-Bus signals
dbus-monitor --system "type='signal',sender='org.crrouter.NetworkControl'"

# Get network state
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetState

# Scan for WiFi networks
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl/WiFi \
  org.crrouter.NetworkControl.WiFi.Scan

# Get WiFi access points
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl/WiFi \
  org.crrouter.NetworkControl.WiFi.GetAccessPoints
```

## Integration with Netctl

The CR D-Bus interface integrates directly with the netctl application. All D-Bus method calls are translated into netctl operations:

1. **Device Control** - Uses netctl's `DeviceController` for device management
2. **WiFi Operations** - Uses netctl's `WifiController` for WiFi operations
3. **VPN Management** - Uses netctl's `VpnManager` for VPN operations

This ensures consistency between the command-line interface and the D-Bus interface.

## Security Considerations

The CR D-Bus service runs on the system bus and requires appropriate D-Bus policy configuration to restrict access to authorized users. By default, the service should be accessible only to:

- Root user
- Users in the `netdev` group
- System services

Create a D-Bus policy file at `/etc/dbus-1/system.d/org.crrouter.NetworkControl.conf`:

```xml
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"/>
    <allow receive_sender="org.crrouter.NetworkControl"/>
  </policy>

  <policy group="netdev">
    <allow send_destination="org.crrouter.NetworkControl"/>
    <allow receive_sender="org.crrouter.NetworkControl"/>
  </policy>

  <policy context="default">
    <deny send_destination="org.crrouter.NetworkControl"/>
  </policy>
</busconfig>
```

## See Also

- [NetworkManager D-Bus API](https://networkmanager.dev/docs/api/latest/)
- [netctl Documentation](../README.md)
- [libnm-compatible CR API](LIBNM_COMPAT_API.md)
