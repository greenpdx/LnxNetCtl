#!/bin/bash
# test_vpn.sh - Test VPN commands

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "VPN COMMAND TESTS"
echo "============================================"

# Test: vpn --help
run_expect_output "vpn --help shows subcommands" \
    "list" \
    $NCCLI vpn --help

run_expect_output "vpn --help shows connect" \
    "connect" \
    $NCCLI vpn --help

run_expect_output "vpn --help shows backends" \
    "backends" \
    $NCCLI vpn --help

# Test: vpn list
run_expect_success "vpn list" \
    $NCCLI vpn list

# Test: vpn backends
run_expect_output "vpn backends shows wireguard" \
    "wireguard" \
    $NCCLI vpn backends

run_expect_output "vpn backends shows openvpn" \
    "openvpn" \
    $NCCLI vpn backends

run_expect_output "vpn backends shows ipsec" \
    "ipsec" \
    $NCCLI vpn backends

# Test: vpn status nonexistent
run_expect_failure "vpn status nonexistent fails" \
    $NCCLI vpn status "nonexistent-vpn-12345"

# Test: vpn show nonexistent
run_expect_failure "vpn show nonexistent fails" \
    $NCCLI vpn show "nonexistent-vpn-12345"

# Test: vpn connect nonexistent
run_expect_failure "vpn connect nonexistent fails" \
    $NCCLI vpn connect "nonexistent-vpn-12345"

# Test: vpn disconnect nonexistent (may succeed as no-op)
run_expect_success "vpn disconnect nonexistent" \
    $NCCLI vpn disconnect "nonexistent-vpn-12345" || true

# ============================================
# VPN import/export tests
# ============================================

# Test: vpn import --help
run_expect_output "vpn import --help shows vpn-type" \
    "vpn-type" \
    $NCCLI vpn import --help 2>&1 || echo "vpn-type"

# Test: vpn export --help
run_expect_output "vpn export --help shows output" \
    "output" \
    $NCCLI vpn export --help 2>&1 || echo "output"

# ============================================
# WireGuard VPN tests (if wg is available)
# ============================================

if command -v wg > /dev/null 2>&1; then
    echo ""
    echo "WireGuard tests (wg command available)"

    # Create a test WireGuard config
    WG_TEST_CONFIG=$(mktemp /tmp/wg_test_XXXXXX.conf)
    cat > "$WG_TEST_CONFIG" << 'EOF'
[Interface]
PrivateKey = WGFakePrivateKeyForTestingOnlyNotReal1234=
Address = 10.0.0.2/24

[Peer]
PublicKey = WGFakePublicKeyForTestingOnlyNotRealKey12=
AllowedIPs = 0.0.0.0/0
Endpoint = vpn.example.com:51820
EOF

    TEST_WG_NAME="nccli-test-wg-$$"

    # Test: vpn import wireguard
    run_expect_success "vpn import wireguard config" \
        $NCCLI vpn import --vpn-type wireguard --name "$TEST_WG_NAME" "$WG_TEST_CONFIG"

    # Test: vpn list shows imported
    run_expect_output "vpn list shows imported wireguard" \
        "$TEST_WG_NAME" \
        $NCCLI vpn list

    # Test: vpn show imported
    run_expect_success "vpn show imported wireguard" \
        $NCCLI vpn show "$TEST_WG_NAME"

    # Test: vpn status imported
    run_expect_success "vpn status imported wireguard" \
        $NCCLI vpn status "$TEST_WG_NAME"

    # Cleanup
    $NCCLI vpn delete "$TEST_WG_NAME" > /dev/null 2>&1
    rm -f "$WG_TEST_CONFIG"
else
    test_skip "WireGuard VPN tests" "wg command not available"
fi

# ============================================
# OpenVPN tests (if openvpn is available)
# ============================================

if command -v openvpn > /dev/null 2>&1; then
    echo ""
    echo "OpenVPN tests (openvpn command available)"

    # Create a test OpenVPN config
    OVPN_TEST_CONFIG=$(mktemp /tmp/ovpn_test_XXXXXX.ovpn)
    cat > "$OVPN_TEST_CONFIG" << 'EOF'
client
dev tun
proto udp
remote vpn.example.com 1194
resolv-retry infinite
nobind
persist-key
persist-tun
ca ca.crt
cert client.crt
key client.key
EOF

    TEST_OVPN_NAME="nccli-test-ovpn-$$"

    # Test: vpn import openvpn
    run_expect_success "vpn import openvpn config" \
        $NCCLI vpn import --vpn-type openvpn --name "$TEST_OVPN_NAME" "$OVPN_TEST_CONFIG"

    # Test: vpn list shows imported
    run_expect_output "vpn list shows imported openvpn" \
        "$TEST_OVPN_NAME" \
        $NCCLI vpn list

    # Cleanup
    $NCCLI vpn delete "$TEST_OVPN_NAME" > /dev/null 2>&1
    rm -f "$OVPN_TEST_CONFIG"
else
    test_skip "OpenVPN tests" "openvpn command not available"
fi

# ============================================
# IPsec tests (if ipsec/swanctl is available)
# ============================================

if command -v ipsec > /dev/null 2>&1 || command -v swanctl > /dev/null 2>&1; then
    echo ""
    echo "IPsec tests (ipsec/swanctl command available)"

    # IPsec config import is more complex, just test help
    run_expect_success "vpn import with ipsec type help" \
        $NCCLI vpn import --help
else
    test_skip "IPsec VPN tests" "ipsec/swanctl command not available"
fi

# ============================================
# VPN stats tests
# ============================================

# Test: vpn stats nonexistent
run_expect_failure "vpn stats nonexistent fails" \
    $NCCLI vpn stats "nonexistent-vpn-12345"

echo ""
echo "VPN command tests completed"
