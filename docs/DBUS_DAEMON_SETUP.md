# D-Bus Daemon Setup Guide

## Overview

This guide explains how to set up the nccli D-Bus daemon to run without permission errors. The daemon provides a D-Bus interface for network management compatible with both our custom CR interface and NetworkManager.

## The Problem

When first running the daemon, you'll encounter this error:

```
WARN: Failed to request D-Bus name: org.freedesktop.DBus.Error.AccessDenied:
Connection ":1.72" is not allowed to own the service "org.crrouter.NetworkControl"
due to security policies in the configuration file.
```

**Why this happens:**
- D-Bus system bus has security policies that control which processes can own specific service names
- By default, only privileged processes or those explicitly granted permission can claim well-known service names
- Without a policy file, the daemon can register interfaces but cannot claim the service name

## The Solution

### Step 1: Create D-Bus Policy File

Create `/etc/dbus-1/system.d/org.crrouter.NetworkControl.conf`:

```xml
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <!-- Allow root to own the service -->
  <policy user="root">
    <allow own="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"/>
  </policy>

  <!-- Allow specific user to own the service (replace 'username' with your user) -->
  <policy user="username">
    <allow own="org.crrouter.NetworkControl"/>
    <allow send_destination="org.crrouter.NetworkControl"/>
  </policy>

  <!-- Allow anyone to send messages to the service -->
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
```

**Important:** Replace `username` with your actual username (e.g., `svvs`, `john`, etc.)

### Step 2: Install the Policy File

```bash
# Create the policy file
sudo nano /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf
# (paste the content above)

# Set correct permissions (must be readable by dbus daemon)
sudo chmod 644 /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf

# Verify ownership
ls -l /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf
# Should show: -rw-r--r-- 1 root root
```

### Step 3: Restart D-Bus Daemon

**Critical:** A simple reload is NOT sufficient. The D-Bus daemon must be fully restarted to load new policy files.

```bash
# Method 1: Restart D-Bus service (preferred)
sudo systemctl restart dbus

# Method 2: Reboot (safest if you're unsure)
sudo reboot
```

**Warning:** Restarting D-Bus may briefly disconnect existing D-Bus-dependent services. On desktop systems, this might require you to log out/in. On headless systems, it's generally safe.

### Step 4: Start the Daemon

```bash
# Build the daemon
cargo build --release --bin nccli

# Run the daemon
./target/release/nccli daemon --cr-dbus

# Or for both CR and NetworkManager compatibility:
./target/release/nccli daemon --cr-dbus --nm-compat
```

### Step 5: Verify Success

You should see this message in the logs:

```
INFO  Successfully registered D-Bus service: org.crrouter.NetworkControl
```

**NOT** the previous error message about AccessDenied.

#### Test the Service

```bash
# Check if service is registered
dbus-send --system --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep crrouter

# Should output:
# string "org.crrouter.NetworkControl"

# Test calling a method
dbus-send --system --print-reply \
  --dest=org.crrouter.NetworkControl \
  /org/crrouter/NetworkControl \
  org.crrouter.NetworkControl.GetVersion

# Should output:
# string "0.1.0"
```

## Understanding the Policy File

### Policy Elements

- `<policy user="username">`: Grants permissions to a specific user
- `<policy user="root">`: Grants permissions to root user
- `<policy context="default">`: Default permissions for all users

### Permission Types

- `<allow own="...">`: Allows owning/claiming a service name
- `<allow send_destination="...">`: Allows sending messages to a service
- `<allow send_interface="...">`: Allows calling methods on specific interfaces

### Security Considerations

**What the policy allows:**
- Specified users can start and own the service
- Any user can call methods on the service (read-only operations)

**What to be careful about:**
- Don't use `<policy context="default">` with `<allow own=...>` - this would let anyone claim the service name
- Only grant ownership to trusted users (root + specific user account)
- The service should validate all inputs and require authentication for privileged operations

## Troubleshooting

### Policy file not being loaded

**Symptom:** Still getting AccessDenied after creating policy file

**Solutions:**
1. Check file permissions: `ls -l /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf`
   - Must be readable: `chmod 644`
2. Check file location: Must be in `/etc/dbus-1/system.d/`
3. Check XML syntax: Use `xmllint` to validate
4. **Restart D-Bus** (reload is not enough): `sudo systemctl restart dbus`

### Service still shows AccessDenied

**Symptom:** Log shows `Connection ":1.XX" is not allowed to own the service`

**Check:**
1. Username in policy file matches your user: `whoami`
2. D-Bus was fully restarted (not just reloaded)
3. No typos in service name in policy file
4. Policy file has correct permissions (644)

### Permission denied reading policy file

**Symptom:** `cat: /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf: Permission denied`

**Solution:**
```bash
sudo chmod 644 /etc/dbus-1/system.d/org.crrouter.NetworkControl.conf
```

### D-Bus restart fails or hangs

**Symptom:** `systemctl restart dbus` fails or system becomes unresponsive

**Solution:**
- On desktop systems, log out and back in
- On server systems, reboot: `sudo reboot`
- D-Bus is a critical system service; rebooting is safest

## Alternative: Running with sudo

If you don't want to create a policy file, you can run the daemon as root:

```bash
sudo ./target/release/nccli daemon --cr-dbus
```

**Pros:**
- No policy file needed
- Works immediately

**Cons:**
- Running as root increases security risk
- Not recommended for production
- Daemon has unnecessary privileges

## Production Deployment

For production systems, create a systemd service:

### Create Service File

`/etc/systemd/system/nccli-daemon.service`:

```ini
[Unit]
Description=nccli Network Control Daemon
After=network.target dbus.service
Requires=dbus.service

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/nccli daemon --cr-dbus --nm-compat
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Install and Enable

```bash
# Install the binary
sudo cp target/release/nccli /usr/local/bin/

# Install the policy file (as shown above)
sudo cp org.crrouter.NetworkControl.conf /etc/dbus-1/system.d/

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable nccli-daemon
sudo systemctl start nccli-daemon

# Check status
sudo systemctl status nccli-daemon
```

## Testing

Run the comprehensive test suite to verify everything works:

```bash
# Test basic D-Bus connectivity
cargo run --example test_dbus_connection

# Expected output should include:
# ✓ SUCCESS: org.crrouter.NetworkControl is running

# Run full test suite
cargo test --test dbus_comprehensive_test -- --nocapture
```

## Summary

**Essential steps for fixing AccessDenied:**

1. ✅ Create policy file in `/etc/dbus-1/system.d/`
2. ✅ Set permissions to 644
3. ✅ Replace `username` with your actual user
4. ✅ **Restart D-Bus** (not reload)
5. ✅ Start daemon
6. ✅ Verify with `dbus-send`

**Success indicators:**
- Log shows: `Successfully registered D-Bus service`
- `dbus-send ListNames` shows `org.crrouter.NetworkControl`
- Method calls work: `GetVersion`, `GetDevices`, etc.

## References

- [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html)
- [D-Bus Security Policies](https://dbus.freedesktop.org/doc/dbus-daemon.1.html#configuration_file)
- [systemd D-Bus Integration](https://www.freedesktop.org/software/systemd/man/sd-bus.html)
