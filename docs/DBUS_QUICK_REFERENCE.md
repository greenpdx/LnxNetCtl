# D-Bus Daemon - Quick Reference Card

## TL;DR - Fix "AccessDenied" Error

```bash
# 1. Create policy file
sudo tee /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf > /dev/null << 'EOF'
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"/>
  </policy>
  <policy user="$(whoami)">
    <allow own="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"/>
  </policy>
  <policy context="default">
    <allow send_destination="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"
           send_interface="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"
           send_interface="org.crrouter.NetworkControl.WiFi"/>
    <allow send_destination="org.crrouter.NetworkControl"
           send_interface="org.crrouter.NetworkControl.VPN"/>
    <allow send_destination="org.crrouter.NetworkControl"
           send_interface="org.freedesktop.DBus.Properties"/>
    <allow send_destination="org.crrouter.NetworkControl"
           send_interface="org.freedesktop.DBus.Introspectable"/>
  </policy>
</busconfig>
EOF

# 2. Fix permissions
sudo chmod 644 /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf

# 3. Restart D-Bus (REQUIRED - reload won't work!)
sudo systemctl restart dbus

# 4. Start daemon
cargo build --release --bin nccli
./target/release/nccli daemon --cr-dbus
```

## Verify It's Working

```bash
# Check service is registered
dbus-send --system --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep crrouter
# Output: string "org.crrouter.NetworkControl"

# Test a method
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetVersion
# Output: string "0.1.0"

# Get devices
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetDevices
```

## Success Indicators

✅ **Working:**
```
INFO Successfully registered D-Bus service: org.crrouter.NetworkControl
```

❌ **Not Working:**
```
WARN Failed to request D-Bus name: org.freedesktop.DBus.Error.AccessDenied
```

## Common Commands

### Start Daemon

```bash
# CR D-Bus only
./target/release/nccli daemon --cr-dbus

# CR + NetworkManager compatibility
./target/release/nccli daemon --cr-dbus --nm-compat

# As background process
nohup ./target/release/nccli daemon --cr-dbus &> /var/log/nccli-daemon.log &
```

### Check Status

```bash
# Check if running
ps aux | grep "nccli daemon"

# Check D-Bus registration
dbus-send --system --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep -i crrouter

# Check logs (if using systemd)
sudo journalctl -u nccli-daemon -f
```

### Stop Daemon

```bash
# Kill by process name
pkill -f "nccli daemon"

# Or if using systemd
sudo systemctl stop nccli-daemon
```

## Troubleshooting Checklist

- [ ] Policy file exists: `/etc/dbus-1/system.d/org.crrouter.NetworkControl.conf`
- [ ] Policy file has correct permissions: `chmod 644`
- [ ] Username in policy matches your user: `whoami`
- [ ] **D-Bus was restarted** (not reloaded): `systemctl restart dbus`
- [ ] No XML syntax errors in policy file
- [ ] Daemon was started after D-Bus restart

## Test Commands

```bash
# Run comprehensive tests
cargo test --test dbus_comprehensive_test -- --nocapture

# Run connectivity tests
cargo run --example test_dbus_connection

# Run example dbus test
cargo run --example dbus_test -- --mode mock
```

## Files Location Reference

| File | Location | Purpose |
|------|----------|---------|
| Policy file | `/etc/dbus-1/system.d/org.crrouter.NetworkControl.conf` | D-Bus permissions |
| Binary | `./target/release/nccli` | Compiled daemon |
| Systemd service | `/etc/systemd/system/nccli-daemon.service` | Service definition |
| Logs | `journalctl -u nccli-daemon` | Runtime logs |

## Available D-Bus Interfaces

- `org.crrouter.NetworkControl` - Main network control interface
- `org.crrouter.NetworkControl.WiFi` - WiFi management
- `org.crrouter.NetworkControl.VPN` - VPN management
- Device paths: `/org/crrouter/NetworkControl/Devices/{interface}`

## See Also

- Full documentation: `docs/DBUS_DAEMON_SETUP.md`
- D-Bus test guide: `docs/DBUS_TEST_GUIDE.md`
- API documentation: `docs/api/`
