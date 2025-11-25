# Tor Plugin for netctld

## Overview

Create a standalone Tor plugin (`netctl-tor`) that integrates with netctld via D-Bus, providing Tor VPN functionality without bloating the main binaries.

## Architecture

```
┌─────────────┐     D-Bus      ┌──────────────┐
│   nccli     │ ◄────────────► │   netctld    │
└─────────────┘                └──────┬───────┘
                                      │ D-Bus
                                      ▼
                               ┌──────────────┐
                               │  netctl-tor  │
                               │   (plugin)   │
                               └──────┬───────┘
                                      │
                                      ▼
                               ┌──────────────┐
                               │  Arti/Tor    │
                               └──────────────┘
```

## Components

### 1. netctl-tor Binary (New)

A standalone daemon that:
- Registers D-Bus service: `org.crrouter.NetworkControl.Tor`
- Path: `/org/crrouter/NetworkControl/Tor`
- Uses arti-client for Tor connectivity
- Runs as a separate systemd service

### 2. D-Bus Interface

**Service**: `org.crrouter.NetworkControl.Tor`

**Methods**:
| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Connect` | `config_path: String` | `bool` | Start Tor connection |
| `Disconnect` | - | `bool` | Stop Tor connection |
| `GetStatus` | - | `Dict` | Get connection status |
| `GetCircuitInfo` | - | `Array<Dict>` | Get current circuits |
| `NewIdentity` | - | `bool` | Request new Tor identity |
| `GetBootstrapProgress` | - | `u8` | Bootstrap percentage (0-100) |
| `SetExitCountry` | `country: String` | `bool` | Set preferred exit node country |
| `GetExitCountry` | - | `String` | Get current exit country |

**Signals**:
| Signal | Parameters | Description |
|--------|------------|-------------|
| `StatusChanged` | `status: u32` | Connection state changed |
| `BootstrapProgress` | `progress: u8` | Bootstrap progress update |
| `CircuitEstablished` | `circuit_id: String` | New circuit established |
| `Error` | `code: u32, message: String` | Error occurred |

**Status Enum**:
```
0 = Disconnected
1 = Bootstrapping
2 = Connected
3 = Disconnecting
4 = Error
```

### 3. nccli Integration

Add Tor subcommands to nccli that communicate via D-Bus:

```bash
nccli tor connect [--config PATH]
nccli tor disconnect
nccli tor status
nccli tor new-identity
nccli tor circuits
nccli tor set-exit-country <COUNTRY>
```

### 4. Systemd Service

`systemd/netctl-tor.service`:
```ini
[Unit]
Description=netctl Tor Plugin
After=netctld.service
Requires=netctld.service

[Service]
Type=simple
ExecStart=/usr/bin/netctl-tor
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Implementation Steps

### Phase 1: Core Plugin Structure

1. Create `src/bin/netctl-tor.rs` - main binary
2. Create `src/tor_plugin/` module:
   - `mod.rs` - module exports
   - `dbus.rs` - D-Bus interface implementation
   - `controller.rs` - Tor connection management via arti
   - `config.rs` - Configuration handling

### Phase 2: D-Bus Interface

1. Define D-Bus interface with zbus
2. Implement methods for connect/disconnect/status
3. Add signal emission for state changes
4. Create D-Bus policy file

### Phase 3: Arti Integration

1. Wrap arti-client for connection management
2. Implement bootstrap progress tracking
3. Handle circuit management
4. Support new identity requests

### Phase 4: nccli Commands

1. Add `tor` subcommand group
2. Implement D-Bus client calls
3. Format output for CLI display

### Phase 5: Packaging

1. Add netctl-tor binary to Cargo.toml
2. Create systemd service file
3. Add D-Bus configuration files
4. Update debian package assets

## File Structure

```
src/
├── bin/
│   └── netctl-tor.rs          # New plugin binary
├── tor_plugin/
│   ├── mod.rs
│   ├── dbus.rs                # D-Bus interface
│   ├── controller.rs          # Tor management
│   └── config.rs              # Config handling
dbus/
├── org.crrouter.NetworkControl.Tor.conf
└── org.crrouter.NetworkControl.Tor.service
systemd/
└── netctl-tor.service
```

## Dependencies (netctl-tor only)

The vpn-tor feature flag will be required only for the netctl-tor binary:
- arti-client
- tor-rtcompat
- zbus (for D-Bus)
- tokio (async runtime)

## Build Commands

```bash
# Build main binaries (small, no Tor)
cargo build --release

# Build Tor plugin (larger, includes arti)
cargo build --release --bin netctl-tor --features vpn-tor
```

## Configuration

Default config location: `/etc/netctl/tor.conf`

```toml
[tor]
# Data directory for Tor state
data_dir = "/var/lib/netctl/tor"

# SOCKS proxy port
socks_port = 9050

# Control port (optional)
control_port = 9051

# Preferred exit countries (ISO 3166-1 alpha-2)
exit_countries = ["US", "DE", "NL"]

# Bridge configuration (optional)
# bridges = ["obfs4 ..."]

# Logging level
log_level = "info"
```

## Security Considerations

1. netctl-tor runs as a separate process with minimal privileges
2. D-Bus policy restricts who can control Tor connections
3. Tor data stored in `/var/lib/netctl/tor` with restricted permissions
4. No secrets stored in main netctl configuration
