# CRRouter Web API Documentation

The CRRouter Web API provides a comprehensive RESTful interface for network device management, DHCP testing, and WiFi operations.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [API Endpoints](#api-endpoints)
  - [Health & Info](#health--info)
  - [Device Management](#device-management)
  - [DHCP Testing](#dhcp-testing)
  - [Interface Management](#interface-management)
  - [WiFi Operations](#wifi-operations)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Overview

The CRRouter Web API is a clean, RESTful interface built on top of the netctl library. It provides:

- **Device Management**: Discover, configure, and monitor network devices
- **DHCP Testing**: Comprehensive DHCP protocol testing and diagnostics
- **WiFi Operations**: Scan and manage WiFi networks
- **Interface Control**: Legacy interface management API

### Key Features

- Clean separation between API layer and business logic
- Comprehensive error handling with proper HTTP status codes
- Self-documenting API with `/api` endpoint
- CORS-enabled for web application integration
- Request tracing and logging

## Architecture

```
┌─────────────────────────────────────┐
│   External Applications             │
│   (Web UI, CLI tools, etc.)         │
└─────────────┬───────────────────────┘
              │ HTTP/REST
┌─────────────▼───────────────────────┐
│   CRRouter Web API (crrouter-web)   │
│   - Request routing                 │
│   - Error handling                  │
│   - JSON serialization              │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│   netctl Library                    │
│   - DeviceController                │
│   - InterfaceController             │
│   - WifiController                  │
│   - DhcpmController                 │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│   System Layer                      │
│   - ip commands                     │
│   - sysfs                           │
│   - netlink                         │
└─────────────────────────────────────┘
```

## Getting Started

### Starting the Server

```bash
# Build the server
cargo build --release --bin crrouter-web

# Run with default settings (port 3000)
./target/release/crrouter-web

# Run with custom port
CRROUTER_WEB_PORT=8080 ./target/release/crrouter-web

# Enable debug logging
RUST_LOG=debug ./target/release/crrouter-web
```

### Base URL

By default, the API runs on `http://0.0.0.0:3000`

### Quick Test

```bash
# Check if the API is running
curl http://localhost:3000/health

# Get API documentation
curl http://localhost:3000/api

# List all devices
curl http://localhost:3000/api/devices
```

## API Endpoints

### Health & Info

#### GET /health

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "service": "crrouter-web",
  "version": "0.1.0"
}
```

#### GET /api

Get comprehensive API documentation and endpoint listing.

**Response:**
```json
{
  "name": "CRRouter Web API",
  "version": "0.1.0",
  "description": "Network device management and control API",
  "endpoints": { ... }
}
```

### Device Management

The device management API provides comprehensive control over network devices.

#### GET /api/devices

List all network devices with full information.

**Query Parameters:**
- `type` (optional): Filter by device type
  - Supported types: `wifi`, `ethernet`, `loopback`, `bridge`, `vlan`, `tuntap`, `veth`, `bond`, `vpn`, `container`, `ppp`

**Examples:**
```bash
# List all devices
curl http://localhost:3000/api/devices

# List only WiFi devices
curl http://localhost:3000/api/devices?type=wifi

# List only Ethernet devices
curl http://localhost:3000/api/devices?type=ethernet
```

**Response:**
```json
[
  {
    "name": "eth0",
    "index": 2,
    "device_type": "ethernet",
    "state": "up",
    "mac_address": "00:11:22:33:44:55",
    "mtu": 1500,
    "addresses": ["192.168.1.100/24"],
    "driver": "e1000e",
    "vendor": "0x8086",
    "model": "0x15b7",
    "bus_info": "0000:00:1f.6",
    "capabilities": {
      "wifi": false,
      "access_point": false,
      "bridge": true,
      "vlan": true,
      "wake_on_lan": true
    },
    "flags": ["UP", "BROADCAST", "RUNNING", "MULTICAST"],
    "stats": {
      "rx_bytes": 1234567,
      "rx_packets": 9876,
      "rx_errors": 0,
      "rx_dropped": 0,
      "tx_bytes": 7654321,
      "tx_packets": 6789,
      "tx_errors": 0,
      "tx_dropped": 0
    },
    "parent": null,
    "children": []
  }
]
```

#### GET /api/devices/:name

Get detailed information about a specific device.

**Path Parameters:**
- `name`: Device name (e.g., "eth0", "wlan0")

**Example:**
```bash
curl http://localhost:3000/api/devices/eth0
```

**Response:** Same structure as individual device in list response.

#### PATCH /api/devices/:name

Configure device settings.

**Path Parameters:**
- `name`: Device name

**Request Body:**
```json
{
  "state": "up",           // Optional: "up" or "down"
  "mtu": 1500,             // Optional: Set MTU
  "mac_address": "00:11:22:33:44:55",  // Optional: Set MAC address
  "add_addresses": [       // Optional: IP addresses to add
    "192.168.1.100/24",
    "fd00::1/64"
  ],
  "remove_addresses": [    // Optional: IP addresses to remove
    "192.168.1.50/24"
  ]
}
```

**Example:**
```bash
# Bring device up
curl -X PATCH http://localhost:3000/api/devices/eth0 \
  -H "Content-Type: application/json" \
  -d '{"state": "up"}'

# Set MTU
curl -X PATCH http://localhost:3000/api/devices/eth0 \
  -H "Content-Type: application/json" \
  -d '{"mtu": 9000}'

# Add IP address
curl -X PATCH http://localhost:3000/api/devices/eth0 \
  -H "Content-Type: application/json" \
  -d '{"add_addresses": ["192.168.1.100/24"]}'

# Multiple operations
curl -X PATCH http://localhost:3000/api/devices/eth0 \
  -H "Content-Type: application/json" \
  -d '{
    "state": "up",
    "mtu": 1500,
    "add_addresses": ["192.168.1.100/24"]
  }'
```

**Response:**
```json
{
  "status": "ok",
  "device": "eth0",
  "message": "Device configured successfully"
}
```

#### DELETE /api/devices/:name

Delete a virtual device (bridge, vlan, veth, tuntap, bond).

**Path Parameters:**
- `name`: Device name

**Example:**
```bash
curl -X DELETE http://localhost:3000/api/devices/veth0
```

**Response:**
```json
{
  "status": "ok",
  "device": "veth0",
  "message": "Device deleted successfully"
}
```

**Error Response (for physical devices):**
```json
{
  "error": "Cannot delete physical device: eth0"
}
```

#### GET /api/devices/:name/stats

Get device statistics.

**Path Parameters:**
- `name`: Device name

**Example:**
```bash
curl http://localhost:3000/api/devices/eth0/stats
```

**Response:**
```json
{
  "device": "eth0",
  "stats": {
    "rx_bytes": 1234567890,
    "rx_packets": 987654,
    "rx_errors": 0,
    "rx_dropped": 12,
    "tx_bytes": 9876543210,
    "tx_packets": 654321,
    "tx_errors": 0,
    "tx_dropped": 5
  }
}
```

### DHCP Testing

The DHCP testing API allows you to send DHCP messages and test DHCP server responses.

#### POST /api/dhcp/test

Run a DHCP test with specified message type.

**Request Body:**
```json
{
  "interface": "eth0",
  "message_type": "Discover",  // Discover, Request, Release, Inform
  "timeout": 5000,             // Optional: Timeout in milliseconds
  "mac_address": null          // Optional: Override MAC address
}
```

**Example:**
```bash
curl -X POST http://localhost:3000/api/dhcp/test \
  -H "Content-Type: application/json" \
  -d '{
    "interface": "eth0",
    "message_type": "Discover"
  }'
```

**Response:**
```json
{
  "success": true,
  "message_type": "Offer",
  "server_ip": "192.168.1.1",
  "offered_ip": "192.168.1.100",
  "subnet_mask": "255.255.255.0",
  "router": "192.168.1.1",
  "dns_servers": ["8.8.8.8", "8.8.4.4"],
  "lease_time": 86400,
  "response_time_ms": 23
}
```

#### POST /api/dhcp/discover

Send a DHCP Discover message.

**Request Body:**
```json
{
  "interface": "eth0",
  "timeout": 5000
}
```

#### POST /api/dhcp/request

Send a DHCP Request message.

#### POST /api/dhcp/release

Send a DHCP Release message.

#### GET /api/dhcp/test-sequence/:interface

Run a full DHCP test sequence (Discover → Request → Release).

**Path Parameters:**
- `interface`: Interface name

**Example:**
```bash
curl http://localhost:3000/api/dhcp/test-sequence/eth0
```

**Response:**
```json
[
  {
    "success": true,
    "message_type": "Offer",
    ...
  },
  {
    "success": true,
    "message_type": "Ack",
    ...
  },
  {
    "success": true,
    "message_type": "Release",
    ...
  }
]
```

### Interface Management

Legacy API for interface management. Prefer using the Device Management API for new applications.

#### GET /api/interfaces

List all network interface names.

**Response:**
```json
["lo", "eth0", "wlan0", "docker0"]
```

#### GET /api/interfaces/:interface

Get interface information.

**Response:**
```json
{
  "name": "eth0",
  "state": "UP",
  "mac_address": "00:11:22:33:44:55",
  "mtu": 1500,
  "addresses": [...],
  "flags": ["UP", "BROADCAST", "RUNNING"]
}
```

### WiFi Operations

#### GET /api/wifi/scan/:interface

Scan for WiFi networks on the specified interface.

**Path Parameters:**
- `interface`: WiFi interface name (e.g., "wlan0")

**Example:**
```bash
curl http://localhost:3000/api/wifi/scan/wlan0
```

**Response:**
```json
{
  "interface": "wlan0",
  "networks": [
    {
      "ssid": "MyNetwork",
      "bssid": "AA:BB:CC:DD:EE:FF",
      "signal": -45,
      "frequency": 2437,
      "capabilities": ["WPA2-PSK", "ESS"]
    },
    {
      "ssid": "AnotherNetwork",
      "bssid": "11:22:33:44:55:66",
      "signal": -67,
      "frequency": 5180,
      "capabilities": ["WPA2-PSK", "WPA3-SAE", "ESS"]
    }
  ]
}
```

## Data Models

### Device

```typescript
interface Device {
  name: string;
  index?: number;
  device_type: DeviceType;
  state: DeviceState;
  mac_address?: string;
  mtu?: number;
  addresses: string[];
  driver?: string;
  vendor?: string;
  model?: string;
  bus_info?: string;
  capabilities: DeviceCapabilities;
  flags: string[];
  stats?: DeviceStats;
  parent?: string;
  children: string[];
}
```

### DeviceType

Enum values: `wifi`, `ethernet`, `loopback`, `bridge`, `vlan`, `tuntap`, `veth`, `bond`, `vpn`, `container`, `ppp`, `unknown`

### DeviceState

Enum values: `up`, `down`, `unmanaged`, `unavailable`, `error`, `unknown`

### DeviceCapabilities

```typescript
interface DeviceCapabilities {
  wifi: boolean;
  access_point: boolean;
  bridge: boolean;
  vlan: boolean;
  mtu_range?: [number, number];
  speeds: number[];
  wake_on_lan: boolean;
}
```

### DeviceStats

```typescript
interface DeviceStats {
  rx_bytes: number;
  rx_packets: number;
  rx_errors: number;
  rx_dropped: number;
  tx_bytes: number;
  tx_packets: number;
  tx_errors: number;
  tx_dropped: number;
}
```

## Error Handling

The API uses standard HTTP status codes and returns error responses in JSON format.

### HTTP Status Codes

- `200 OK`: Successful request
- `400 Bad Request`: Invalid parameters or malformed request
- `403 Forbidden`: Permission denied
- `404 Not Found`: Device or resource not found
- `409 Conflict`: Resource already exists or invalid state
- `408 Request Timeout`: Operation timed out
- `500 Internal Server Error`: Command execution failed
- `501 Not Implemented`: Feature not supported
- `503 Service Unavailable`: Required service unavailable

### Error Response Format

```json
{
  "error": "Human-readable error message",
  "details": null
}
```

### Examples

```json
// 404 - Device not found
{
  "error": "Interface not found: eth99"
}

// 400 - Invalid parameter
{
  "error": "Unknown device type: foobar"
}

// 403 - Permission denied
{
  "error": "Permission denied: requires root privileges"
}

// 500 - Command failed
{
  "error": "Command 'ip link set dev eth0 up' failed with code 1: RTNETLINK answers: Operation not permitted"
}
```

## Examples

### Complete Device Management Workflow

```bash
#!/bin/bash

# 1. List all devices
echo "=== All Devices ==="
curl -s http://localhost:3000/api/devices | jq .

# 2. Get specific device info
echo -e "\n=== Device eth0 ==="
curl -s http://localhost:3000/api/devices/eth0 | jq .

# 3. Configure device
echo -e "\n=== Configuring eth0 ==="
curl -s -X PATCH http://localhost:3000/api/devices/eth0 \
  -H "Content-Type: application/json" \
  -d '{
    "state": "up",
    "mtu": 1500,
    "add_addresses": ["192.168.100.1/24"]
  }' | jq .

# 4. Get device statistics
echo -e "\n=== Device Statistics ==="
curl -s http://localhost:3000/api/devices/eth0/stats | jq .

# 5. List only WiFi devices
echo -e "\n=== WiFi Devices ==="
curl -s "http://localhost:3000/api/devices?type=wifi" | jq .
```

### DHCP Testing Workflow

```bash
#!/bin/bash

INTERFACE="eth0"

# Run full DHCP test sequence
echo "=== Running DHCP Test Sequence on $INTERFACE ==="
curl -s http://localhost:3000/api/dhcp/test-sequence/$INTERFACE | jq .

# Or run individual tests
echo -e "\n=== DHCP Discover ==="
curl -s -X POST http://localhost:3000/api/dhcp/discover \
  -H "Content-Type: application/json" \
  -d "{\"interface\": \"$INTERFACE\"}" | jq .
```

### WiFi Scanning

```bash
#!/bin/bash

# Scan for WiFi networks
echo "=== Scanning WiFi Networks ==="
curl -s http://localhost:3000/api/wifi/scan/wlan0 | jq '.networks[] | {ssid, signal, frequency}'
```

## Integration Examples

### Python

```python
import requests

# List all devices
response = requests.get('http://localhost:3000/api/devices')
devices = response.json()

for device in devices:
    print(f"{device['name']}: {device['device_type']} - {device['state']}")

# Configure a device
config = {
    'state': 'up',
    'mtu': 1500,
    'add_addresses': ['192.168.1.100/24']
}
response = requests.patch(
    'http://localhost:3000/api/devices/eth0',
    json=config
)
print(response.json())
```

### JavaScript/Node.js

```javascript
// List devices
async function listDevices() {
  const response = await fetch('http://localhost:3000/api/devices');
  const devices = await response.json();
  return devices;
}

// Configure device
async function configureDevice(name, config) {
  const response = await fetch(`http://localhost:3000/api/devices/${name}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config)
  });
  return response.json();
}

// Usage
const devices = await listDevices();
console.log(devices);

await configureDevice('eth0', {
  state: 'up',
  mtu: 1500,
  add_addresses: ['192.168.1.100/24']
});
```

## Security Considerations

- The API requires root/elevated privileges for most operations
- No authentication is built-in; use a reverse proxy for authentication
- CORS is permissive by default; configure appropriately for production
- Validate all input on the client side before sending requests
- Use HTTPS in production environments

## Support

For issues, questions, or contributions, please visit:
- GitHub: https://github.com/your-org/netctl
- Documentation: See `docs/` directory
