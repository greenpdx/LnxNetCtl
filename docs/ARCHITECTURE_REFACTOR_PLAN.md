# Architecture Refactor Plan: Daemon + CLI Separation

## Overview

Split the current monolithic `nccli` into two separate programs:

1. **`netctld`** - Daemon program providing D-Bus services
2. **`netctl`** - CLI program that communicates with daemon via D-Bus

This follows the proven architecture of NetworkManager (daemon) + nmcli (CLI).

## Current Architecture

```
┌─────────────────────────────────┐
│         nccli (binary)          │
│                                 │
│  ┌──────────┐   ┌───────────┐  │
│  │ CLI Cmds │   │  Daemon   │  │
│  └────┬─────┘   └─────┬─────┘  │
│       │               │        │
│       └───────┬───────┘        │
│               │                │
└───────────────┼────────────────┘
                │
        ┌───────▼────────┐
        │  libnetctl     │
        │  (library)     │
        └───────┬────────┘
                │
        ┌───────▼────────┐
        │  System APIs   │
        │ (netlink, etc) │
        └────────────────┘
```

**Issues:**
- CLI commands use library directly (requires privileges)
- Can't have multiple CLI instances safely
- Daemon mode mixed with CLI code
- No separation of concerns

## Target Architecture

```
┌─────────────┐                    ┌──────────────┐
│    netctl   │◄──── D-Bus ───────►│   netctld    │
│  (CLI tool) │                    │   (daemon)   │
└─────────────┘                    └──────┬───────┘
                                          │
                                   ┌──────▼───────┐
                                   │  libnetctl   │
                                   │  (library)   │
                                   └──────┬───────┘
                                          │
                                   ┌──────▼───────┐
                                   │ System APIs  │
                                   └──────────────┘

Multiple CLI instances can connect to single daemon
```

**Benefits:**
- Single daemon with all privileges
- CLI runs as regular user
- Multiple clients supported
- Clean separation of concerns
- Daemon can be managed by systemd
- Clients can be in different languages (Python, etc.)

## Phase 1: Create Daemon Binary (`netctld`)

### 1.1 Create New Binary

**File:** `src/bin/netctld.rs`

Extract daemon functionality from `nccli.rs`:
- D-Bus service setup (CR + NetworkManager compat)
- Network monitoring
- Device discovery
- Event handling

**Remove from daemon:**
- CLI argument parsing (except daemon-specific options)
- User interaction
- Output formatting

### 1.2 Daemon Command Structure

```rust
// src/bin/netctld.rs

#[derive(Parser)]
#[command(name = "netctld")]
#[command(about = "Network Control Daemon")]
struct Args {
    /// Enable CR D-Bus interface
    #[arg(long, default_value = "true")]
    cr_dbus: bool,

    /// Enable NetworkManager compatibility
    #[arg(long, default_value = "false")]
    nm_compat: bool,

    /// Foreground mode (don't daemonize)
    #[arg(short, long)]
    foreground: bool,

    /// PID file location
    #[arg(long, default_value = "/var/run/netctld.pid")]
    pid_file: PathBuf,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}
```

### 1.3 D-Bus Interface Extensions

Ensure D-Bus interfaces support ALL operations:

**Required methods to add/verify:**

```rust
// org.crrouter.NetworkControl
- GetVersion() → String
- GetDevices() → Vec<ObjectPath>
- GetDeviceByInterface(name: String) → ObjectPath
- GetState() → u32
- GetConnectivity() → u32
- EnableNetworking(enable: bool)
- EnableWireless(enable: bool)
- Reload()

// org.crrouter.NetworkControl.WiFi
- GetEnabled() → bool
- SetEnabled(enabled: bool)
- Scan()
- GetAccessPoints() → Vec<ObjectPath>
- GetCurrentSSID() → String
- Connect(ssid: String, password: String) → bool
- Disconnect()

// org.crrouter.NetworkControl.VPN
- GetConnections() → Vec<String>
- GetConnectionInfo(name: String) → HashMap
- Connect(name: String) → bool
- Disconnect(name: String) → bool
- ImportConfig(type: String, file: String, name: String)
- DeleteConnection(name: String)

// org.crrouter.NetworkControl.Device (per-device)
- GetInterface() → String
- GetType() → u32
- GetState() → u32
- GetIp4Config() → HashMap
- GetIp6Config() → HashMap
- Activate()
- Deactivate()
- SetIpAddress(addr: String, prefix: u8)
- AddRoute(dest: String, gateway: String)
```

## Phase 2: Create CLI Binary (`netctl`)

### 2.1 Create New Binary

**File:** `src/bin/netctl.rs`

Pure CLI tool that ONLY communicates via D-Bus.

### 2.2 D-Bus Client Library

**File:** `src/dbus_client.rs`

Create a clean D-Bus client wrapper:

```rust
pub struct NetctlClient {
    connection: Connection,
    proxy: NetworkControlProxy<'static>,
}

impl NetctlClient {
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        let proxy = NetworkControlProxy::builder(&conn)
            .destination("org.crrouter.NetworkControl")?
            .path("/org/crrouter/NetworkControl")?
            .build()
            .await?;
        Ok(Self { connection: conn, proxy })
    }

    pub async fn get_version(&self) -> Result<String> {
        self.proxy.get_version().await
    }

    pub async fn list_devices(&self) -> Result<Vec<String>> {
        self.proxy.get_devices().await
    }

    // ... all other operations
}
```

### 2.3 CLI Command Structure

Keep similar to current `nccli`:

```rust
// src/bin/netctl.rs

#[derive(Parser)]
#[command(name = "netctl")]
enum Commands {
    /// General commands
    #[command(subcommand)]
    General(GeneralCommands),

    /// Device management
    #[command(subcommand)]
    Device(DeviceCommands),

    /// WiFi management
    #[command(subcommand)]
    Wifi(WifiCommands),

    /// VPN management
    #[command(subcommand)]
    Vpn(VpnCommands),

    // ... etc
}

async fn handle_general(cmd: &GeneralCommands, client: &NetctlClient) {
    match cmd {
        GeneralCommands::Status => {
            let state = client.get_state().await?;
            let connectivity = client.get_connectivity().await?;
            println!("State: {}", state);
            println!("Connectivity: {}", connectivity);
        }
        // ... etc
    }
}
```

### 2.4 Migration from Direct Library Calls

**Before (current `nccli`):**
```rust
// Direct library usage
let iface_ctrl = interface::InterfaceController::new();
let interfaces = iface_ctrl.list().await?;
```

**After (`netctl` via D-Bus):**
```rust
// D-Bus call
let client = NetctlClient::new().await?;
let devices = client.list_devices().await?;
```

## Phase 3: Library Refactoring

### 3.1 Split Library Responsibilities

```
libnetctl/
├── core/           # Core functionality (used by daemon)
│   ├── interface.rs
│   ├── wifi.rs
│   ├── vpn.rs
│   └── ...
├── dbus_server/   # D-Bus service implementation (daemon)
│   ├── network_control.rs
│   ├── wifi.rs
│   └── vpn.rs
└── dbus_client/   # D-Bus client (CLI)
    ├── client.rs
    └── types.rs
```

### 3.2 Feature Flags

Add Cargo features for different use cases:

```toml
[features]
default = ["daemon", "client"]
daemon = ["dbus-server", "netlink", "wifi"]
client = ["dbus-client"]
dbus-server = ["zbus", "zbus-macros"]
dbus-client = ["zbus"]
```

## Phase 4: Implementation Steps

### Step 1: Prepare D-Bus Interfaces (Week 1)
- [ ] Audit current D-Bus interfaces in `src/cr_dbus/`
- [ ] Identify missing methods needed for CLI operations
- [ ] Add missing D-Bus methods
- [ ] Write D-Bus interface tests
- [ ] Document all D-Bus APIs

### Step 2: Create Daemon Binary (Week 1-2)
- [ ] Create `src/bin/netctld.rs`
- [ ] Extract daemon code from `nccli.rs`
- [ ] Add daemon-specific options (pid file, daemonize, etc.)
- [ ] Create systemd service file
- [ ] Test daemon standalone

### Step 3: Create D-Bus Client Library (Week 2)
- [ ] Create `src/dbus_client/mod.rs`
- [ ] Implement D-Bus proxy wrappers
- [ ] Add error handling and retries
- [ ] Create type conversions (D-Bus ↔ Rust types)
- [ ] Write client library tests

### Step 4: Create CLI Binary (Week 2-3)
- [ ] Create `src/bin/netctl.rs`
- [ ] Port all commands to use D-Bus client
- [ ] Maintain same CLI interface (backward compatible)
- [ ] Add connection error handling
- [ ] Test all commands end-to-end

### Step 5: Testing & Validation (Week 3)
- [ ] Integration tests (daemon + CLI)
- [ ] Regression tests (ensure all features work)
- [ ] Performance testing
- [ ] Multi-client testing
- [ ] Error condition testing

### Step 6: Migration & Deprecation (Week 4)
- [ ] Update documentation
- [ ] Create migration guide
- [ ] Mark old `nccli daemon` as deprecated
- [ ] Add warnings to old code paths
- [ ] Update build scripts

### Step 7: Cleanup (Week 4)
- [ ] Remove duplicate code
- [ ] Remove `nccli` daemon mode
- [ ] Rename `nccli` → `netctl` (if desired)
- [ ] Clean up unused dependencies
- [ ] Final testing

## Phase 5: Deployment Strategy

### 5.1 Systemd Service

**File:** `/etc/systemd/system/netctld.service`

```ini
[Unit]
Description=Network Control Daemon
Documentation=man:netctld(8)
After=network.target dbus.service
Requires=dbus.service

[Service]
Type=dbus
BusName=org.crrouter.NetworkControl
ExecStart=/usr/local/bin/netctld --foreground
Restart=on-failure
RestartSec=5s

# Security hardening
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/netctl

[Install]
WantedBy=multi-user.target
Alias=dbus-org.crrouter.NetworkControl.service
```

### 5.2 D-Bus Activation

**File:** `/usr/share/dbus-1/system-services/org.crrouter.NetworkControl.service`

```ini
[D-BUS Service]
Name=org.crrouter.NetworkControl
Exec=/usr/local/bin/netctld --foreground
User=root
SystemdService=netctld.service
```

This enables automatic daemon start when CLI is used.

### 5.3 Package Structure

```
netctl-daemon package:
├── /usr/local/bin/netctld
├── /etc/systemd/system/netctld.service
├── /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf
└── /usr/share/dbus-1/system-services/org.crrouter.NetworkControl.service

netctl-cli package:
├── /usr/local/bin/netctl
└── /usr/share/man/man1/netctl.1
```

## Phase 6: Backward Compatibility

### 6.1 Transition Period

Support both architectures during transition:

```rust
// In netctl CLI
if !can_connect_to_daemon() {
    eprintln!("Warning: netctld daemon not running");
    eprintln!("Falling back to direct mode (requires root)");
    eprintln!("Please install and start netctld for best experience");

    // Fallback to direct library usage
    use_direct_library()?;
}
```

### 6.2 Alias and Symlinks

```bash
# Keep nccli as alias to netctl
ln -s /usr/local/bin/netctl /usr/local/bin/nccli
```

## Code Examples

### Example 1: Device List Command

**Before (direct library):**
```rust
async fn handle_device_list() -> Result<()> {
    let ctrl = InterfaceController::new();
    let interfaces = ctrl.list().await?;

    for iface in interfaces {
        let info = ctrl.get_info(&iface).await?;
        println!("{}: {}", iface, info.state.unwrap_or_default());
    }
    Ok(())
}
```

**After (via D-Bus):**
```rust
async fn handle_device_list(client: &NetctlClient) -> Result<()> {
    let devices = client.list_devices().await?;

    for device_path in devices {
        let info = client.get_device_info(&device_path).await?;
        println!("{}: {}",
            info.get("Interface").unwrap_or(&"?".into()),
            info.get("State").unwrap_or(&"?".into())
        );
    }
    Ok(())
}
```

### Example 2: WiFi Connect

**Before:**
```rust
async fn wifi_connect(ssid: &str, password: &str) -> Result<()> {
    let wifi_ctrl = WifiController::new("wlan0");
    wifi_ctrl.connect(ssid, password).await?;
    Ok(())
}
```

**After:**
```rust
async fn wifi_connect(client: &NetctlClient, ssid: &str, password: &str) -> Result<()> {
    client.wifi_connect(ssid, password).await?;
    Ok(())
}
```

## Testing Strategy

### Unit Tests
- D-Bus client library functions
- Command parsing
- Type conversions

### Integration Tests
- Start mock daemon
- Run CLI commands
- Verify D-Bus messages
- Check results

### End-to-End Tests
- Start real daemon
- Run full command suite
- Verify system state changes
- Multi-client scenarios

### Performance Tests
- CLI startup time
- Command latency
- Daemon resource usage
- Concurrent client handling

## Migration Checklist

### For Developers
- [ ] Update build scripts for two binaries
- [ ] Update CI/CD for separate testing
- [ ] Update documentation
- [ ] Create migration guide
- [ ] Version bump (1.0.0 → 2.0.0)

### For Users
- [ ] Install netctld daemon
- [ ] Enable systemd service
- [ ] Update aliases/scripts
- [ ] Test existing workflows
- [ ] Report issues

### For Packagers
- [ ] Split into two packages
- [ ] Add daemon dependencies
- [ ] Update install scripts
- [ ] Test upgrade path

## Timeline

| Week | Tasks | Deliverables |
|------|-------|--------------|
| 1 | D-Bus API audit + daemon extraction | Working netctld binary |
| 2 | D-Bus client library + CLI skeleton | netctl can connect to daemon |
| 3 | Port all commands + testing | Feature-complete netctl |
| 4 | Documentation + migration | Production ready |

## Success Criteria

- [ ] `netctld` runs as systemd service
- [ ] `netctl` works without root privileges
- [ ] All original `nccli` commands work
- [ ] Multiple `netctl` instances can run simultaneously
- [ ] Performance is acceptable (< 100ms command latency)
- [ ] Full test coverage
- [ ] Documentation complete
- [ ] Migration guide available

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Missing D-Bus methods | High | Thorough API audit first |
| Performance degradation | Medium | Benchmark early, optimize |
| Breaking changes for users | High | Maintain compatibility layer |
| Complex testing | Medium | Automated integration tests |
| Daemon crashes | High | Robust error handling + auto-restart |

## Future Enhancements

After split is complete:

1. **Python/Go/Rust client libraries** - Easy to create from D-Bus spec
2. **GUI tool** - Can use same D-Bus interface
3. **Web API** - Daemon can expose HTTP alongside D-Bus
4. **Remote management** - D-Bus over network
5. **Plugin system** - Third-party D-Bus services

## References

- NetworkManager architecture (nmcli + daemon)
- systemd D-Bus activation
- D-Bus specification
- zbus documentation
