# Architecture Diagrams

## Current Architecture (Monolithic)

```
┌─────────────────────────────────────────────────┐
│                    nccli                        │
│  ┌──────────────────┬──────────────────────┐   │
│  │  CLI Commands    │   Daemon Mode        │   │
│  │  ─────────────   │   ───────────        │   │
│  │  • device list   │   • D-Bus service    │   │
│  │  • wifi scan     │   • Network monitor  │   │
│  │  • vpn connect   │   • Event loop       │   │
│  │  • connection    │   • State management │   │
│  │    ... all cmds  │                      │   │
│  └────────┬─────────┴──────────┬───────────┘   │
│           │                    │               │
│           └────────┬───────────┘               │
│                    │                           │
└────────────────────┼───────────────────────────┘
                     │
         ┌───────────▼────────────┐
         │     libnetctl          │
         │  ─────────────────     │
         │  • Interface control   │
         │  • WiFi management     │
         │  • VPN handling        │
         │  • DHCP client         │
         └───────────┬────────────┘
                     │
         ┌───────────▼────────────┐
         │    System APIs         │
         │  ─────────────────     │
         │  • Netlink             │
         │  • ioctl               │
         │  • WPA supplicant      │
         └────────────────────────┘

ISSUES:
✗ CLI requires root (direct system access)
✗ Only one instance at a time
✗ Mixed concerns (CLI + Daemon)
✗ No client/server separation
```

## Target Architecture (Daemon + CLI)

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   netctl     │     │   netctl     │     │  Python/GUI  │
│  (CLI #1)    │     │  (CLI #2)    │     │   Client     │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                   D-Bus (System Bus)
                            │
              ┌─────────────▼─────────────┐
              │    org.crrouter.          │
              │    NetworkControl         │
              └─────────────┬─────────────┘
                            │
                ┌───────────▼───────────┐
                │      netctld          │
                │     (daemon)          │
                │  ───────────────      │
                │  • D-Bus service      │
                │  • Network monitor    │
                │  • Event handling     │
                │  • State management   │
                └───────────┬───────────┘
                            │
                ┌───────────▼───────────┐
                │     libnetctl         │
                │  ───────────────      │
                │  • Core logic only    │
                │  • No CLI code        │
                └───────────┬───────────┘
                            │
                ┌───────────▼───────────┐
                │    System APIs        │
                │  ───────────────      │
                │  • Netlink            │
                │  • ioctl              │
                │  • WPA supplicant     │
                └───────────────────────┘

BENEFITS:
✓ CLI runs as regular user
✓ Multiple clients supported
✓ Clean separation of concerns
✓ Language-agnostic clients
✓ Managed by systemd
```

## Communication Flow

### Example: WiFi Scan Command

```
User runs: netctl wifi scan

┌──────────┐
│   User   │
└────┬─────┘
     │
     │ $ netctl wifi scan
     ▼
┌──────────────────────┐
│  netctl (CLI)        │
│ ─────────────        │
│ 1. Parse command     │
│ 2. Connect to D-Bus  │
│ 3. Call WiFi.Scan()  │
└──────┬───────────────┘
       │
       │ D-Bus Method Call
       │ org.crrouter.NetworkControl.WiFi.Scan()
       ▼
┌────────────────────────┐
│  netctld (daemon)      │
│ ──────────────         │
│ 1. Receive D-Bus call  │
│ 2. Validate request    │
│ 3. Execute wifi scan   │
│ 4. Return results      │
└──────┬─────────────────┘
       │
       │ Calls library
       ▼
┌────────────────────────┐
│  libnetctl             │
│ ──────────────         │
│ wifi::scan()           │
│   └─> wpa_supplicant   │
└──────┬─────────────────┘
       │
       │ Netlink/IPC
       ▼
┌────────────────────────┐
│  WPA Supplicant        │
│  ──────────────        │
│  Performs actual scan  │
└──────┬─────────────────┘
       │
       │ Results
       ▼
     (back up the chain)
       │
       ▼
┌──────────────────────┐
│  User sees output    │
│  SSID       Signal   │
│  MyWiFi     85%      │
│  Neighbor   45%      │
└──────────────────────┘
```

## File Structure Comparison

### Before (Current)

```
netctl/
├── src/
│   ├── bin/
│   │   ├── nccli.rs           ← CLI + Daemon mixed (2500 lines)
│   │   └── nm-converter.rs
│   ├── lib.rs
│   ├── interface.rs
│   ├── wifi.rs
│   ├── vpn.rs
│   ├── cr_dbus/              ← D-Bus server code
│   │   ├── network_control.rs
│   │   ├── wifi.rs
│   │   └── vpn.rs
│   └── ...
└── Cargo.toml

Issues:
- Mixed responsibilities in nccli.rs
- No D-Bus client code
- CLI uses library directly
```

### After (Target)

```
netctl/
├── src/
│   ├── bin/
│   │   ├── netctld.rs        ← Daemon only (~500 lines)
│   │   ├── netctl.rs         ← CLI only (~800 lines)
│   │   └── nm-converter.rs
│   ├── lib.rs
│   ├── core/                 ← Core functionality
│   │   ├── interface.rs
│   │   ├── wifi.rs
│   │   ├── vpn.rs
│   │   └── ...
│   ├── dbus_server/         ← D-Bus service (used by daemon)
│   │   ├── network_control.rs
│   │   ├── wifi.rs
│   │   └── vpn.rs
│   └── dbus_client/         ← D-Bus client (used by CLI)
│       ├── client.rs
│       └── types.rs
└── Cargo.toml

Benefits:
- Clear separation
- D-Bus client library
- Independent binaries
```

## Process Lifecycle

### Daemon (netctld)

```
System Boot
    │
    ▼
systemd starts
    │
    ▼
┌─────────────────────────┐
│ systemd launches        │
│ netctld.service         │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│ netctld process starts  │
│ 1. Initialize logging   │
│ 2. Connect to D-Bus     │
│ 3. Claim service name   │
│ 4. Discover devices     │
│ 5. Start monitoring     │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│ Main event loop         │
│ • Process D-Bus calls   │
│ • Monitor network       │
│ • Emit signals          │
│ • Handle errors         │
└─────────────────────────┘
           │
           │ (runs until stopped)
           │
     System Shutdown
```

### CLI (netctl)

```
User runs command
    │
    ▼
┌─────────────────────────┐
│ netctl process starts   │
│ 1. Parse arguments      │
│ 2. Connect to D-Bus     │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│ Check daemon running    │
└──────────┬──────────────┘
           │
     ┌─────┴─────┐
     │           │
   No│           │Yes
     ▼           ▼
┌─────────┐  ┌──────────┐
│ Error   │  │ Execute  │
│ Exit 1  │  │ Command  │
└─────────┘  └────┬─────┘
                  │
                  ▼
           ┌──────────────┐
           │ Print result │
           └──────┬───────┘
                  │
                  ▼
              Exit 0
```

## Security Model

### Before (Monolithic)

```
┌──────────────────────┐
│  nccli (any user)    │
│  ────────────────    │
│  Requires:           │
│  • CAP_NET_ADMIN     │  ← All users need privileges
│  • Root or sudo      │
└──────────────────────┘
         │
         ▼
┌──────────────────────┐
│  System APIs         │
│  (full access)       │
└──────────────────────┘

Risk: Every user needs elevated privileges
```

### After (Daemon + CLI)

```
┌──────────────────────┐
│  netctl (any user)   │
│  ────────────────    │
│  Requires:           │
│  • No privileges     │  ← Regular user OK
│  • D-Bus access      │
└──────┬───────────────┘
       │
       │ D-Bus (controlled access)
       ▼
┌──────────────────────┐
│  netctld (root)      │
│  ────────────────    │
│  Has:                │  ← Only daemon has privileges
│  • CAP_NET_ADMIN     │
│  • Runs as root      │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  System APIs         │
│  (controlled)        │
└──────────────────────┘

Benefit: Privilege separation, D-Bus policy controls access
```

## Deployment Comparison

### Before

```
Install:
1. Copy nccli binary
2. Done

Usage:
$ sudo nccli device list     # Needs sudo
$ sudo nccli wifi scan       # Needs sudo
```

### After

```
Install:
1. Copy netctld → /usr/local/bin/
2. Copy netctl → /usr/local/bin/
3. Install systemd service
4. Install D-Bus policy
5. Enable & start daemon

Usage:
$ netctl device list         # No sudo needed!
$ netctl wifi scan          # No sudo needed!
$ netctl vpn connect work   # No sudo needed!
```

## Summary

| Aspect | Before | After |
|--------|--------|-------|
| **Binaries** | 1 (nccli) | 2 (netctld + netctl) |
| **Privileges** | CLI needs root | Only daemon needs root |
| **Concurrency** | Single instance | Multiple CLI instances |
| **Architecture** | Monolithic | Client-server |
| **Deployment** | Simple binary | Daemon + CLI + systemd |
| **Complexity** | Low setup, high coupling | Higher setup, low coupling |
| **Extensibility** | Limited | High (any D-Bus client) |
| **Security** | All users need privileges | Centralized privilege control |

## Migration Path

```
Current State                Transition                    End State
─────────────               ──────────                    ─────────

    nccli          →     nccli + netctld + netctl    →    netctld + netctl
  (monolithic)              (coexistence)                  (clean split)
                                   │
                                   │
                      deprecate nccli daemon mode
                      keep nccli as alias to netctl
```
