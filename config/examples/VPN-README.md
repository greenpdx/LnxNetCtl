# VPN Configuration Examples

This directory contains example configurations for different VPN types supported by LnxNetCtl.

## Supported VPN Types

- **WireGuard**: Modern, fast, and secure VPN using state-of-the-art cryptography
- **OpenVPN**: Traditional, widely-supported SSL/TLS VPN
- **IPsec/IKEv2**: Industry-standard IPsec with IKEv2 key exchange (using strongSwan/FreeSWAN/Libreswan)

## Quick Start

### 1. List Available VPN Backends

```bash
netctl vpn backends
```

### 2. Create a VPN Connection

#### Option A: From a TOML configuration file

```bash
netctl vpn create vpn-wireguard.toml
```

#### Option B: Import an existing VPN configuration

```bash
# Import WireGuard .conf file
netctl vpn import --vpn-type wireguard --name "My WireGuard VPN" /path/to/wg0.conf

# Import OpenVPN .ovpn file
netctl vpn import --vpn-type openvpn --name "My OpenVPN" /path/to/client.ovpn

# Import IPsec configuration
netctl vpn import --vpn-type ipsec --name "My IPsec VPN" /path/to/ipsec.conf
```

### 3. Connect to VPN

```bash
netctl vpn connect "WireGuard Home"
```

### 4. Check VPN Status

```bash
netctl vpn status "WireGuard Home"
```

### 5. View Statistics

```bash
netctl vpn stats "WireGuard Home"
```

### 6. Disconnect

```bash
netctl vpn disconnect "WireGuard Home"
```

### 7. List All VPN Connections

```bash
netctl vpn list
```

## Configuration File Format

All VPN configurations use TOML format with the following structure:

```toml
[connection]
uuid = "unique-uuid-here"
name = "Connection Name"
conn_type = "vpn"
autoconnect = false

[connection.settings]
vpn_type = "wireguard|openvpn|ipsec"
# Backend-specific settings...
```

## WireGuard Configuration

See `vpn-wireguard.toml` for a complete example.

### Required Fields:
- `private_key`: Your WireGuard private key
- `peer.public_key`: Server's public key
- `peer.endpoint`: Server address and port

### Generate Keys:

```bash
# Generate private key
wg genkey

# Generate public key from private key
echo "PRIVATE_KEY" | wg pubkey
```

## OpenVPN Configuration

See `vpn-openvpn.toml` for a complete example.

### Two Configuration Methods:

1. **Reference existing .ovpn file:**
   ```toml
   [connection.settings]
   vpn_type = "openvpn"
   config_file = "/path/to/client.ovpn"
   ```

2. **Specify parameters directly:**
   ```toml
   [connection.settings]
   vpn_type = "openvpn"
   remote = "vpn.example.com"
   port = 1194
   proto = "udp"
   ca = "/etc/openvpn/ca.crt"
   cert = "/etc/openvpn/client.crt"
   key = "/etc/openvpn/client.key"
   ```

## IPsec/IKEv2 Configuration

See `vpn-ipsec.toml` for a complete example.

### Required Fields:
- `right`: Remote gateway address
- At least one authentication method (PSK, certificate, EAP, or XAUTH)

### Authentication Methods:

1. **Certificate-based (recommended):**
   ```toml
   leftcert = "client.pem"
   ```

2. **Pre-Shared Key:**
   ```toml
   psk = "shared_secret"
   ```

3. **EAP (username/password):**
   ```toml
   eap_identity = "user@example.com"
   eap_password = "password"
   ```

## Advanced Usage

### Export VPN Configuration

```bash
netctl vpn export "WireGuard Home" --output /path/to/export.conf
```

### Delete VPN Connection

```bash
netctl vpn delete "WireGuard Home"
```

### JSON Output

All commands support JSON output with the `-o json` flag:

```bash
netctl vpn list -o json
netctl vpn status "WireGuard Home" -o json
```

## Troubleshooting

### Check if VPN software is installed:

```bash
# WireGuard
wg --version
wg-quick --version

# OpenVPN
openvpn --version

# IPsec (strongSwan/Libreswan)
ipsec --version
```

### View detailed connection information:

```bash
netctl vpn show "Connection Name"
```

### Enable verbose output:

```bash
netctl -v vpn connect "Connection Name"
```

## Security Notes

1. **Protect your private keys**: Keep private keys, certificates, and passwords secure with appropriate file permissions (0600).

2. **Use strong authentication**: Prefer certificate-based authentication over PSK when possible.

3. **Keep software updated**: Regularly update VPN software to get security patches.

4. **Verify server certificates**: Always verify the authenticity of VPN server certificates.

5. **Use secure key storage**: Consider using a key management system for production deployments.

## Environment Variables

- `NETCTL_CONFIG_DIR`: Directory for VPN configurations (default: `/etc/netctl`)

## See Also

- WireGuard documentation: https://www.wireguard.com/
- OpenVPN documentation: https://openvpn.net/community-resources/
- strongSwan documentation: https://www.strongswan.org/documentation.html
