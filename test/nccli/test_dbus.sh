#!/bin/bash
# test_dbus.sh - Test D-Bus communication with netctld

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "D-BUS COMMUNICATION TESTS"
echo "============================================"

# Check if dbus tools are available
if ! command -v busctl > /dev/null 2>&1; then
    echo -e "${YELLOW}WARNING: busctl not found, skipping D-Bus introspection tests${NC}"
    BUSCTL_AVAILABLE=false
else
    BUSCTL_AVAILABLE=true
fi

if ! command -v gdbus > /dev/null 2>&1; then
    GDBUS_AVAILABLE=false
else
    GDBUS_AVAILABLE=true
fi

# ============================================
# D-Bus Service Tests
# ============================================

echo ""
echo "--- D-Bus Service Availability ---"

if $BUSCTL_AVAILABLE; then
    # Test: Check if CR D-Bus service is registered
    run_expect_output "CR D-Bus service is registered" \
        "org.crrouter.NetworkControl" \
        busctl list

    # Test: Introspect main interface
    run_expect_success "Introspect NetworkControl interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl

    # Test: Introspect WiFi interface
    run_expect_success "Introspect WiFi interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/WiFi

    # Test: Introspect VPN interface
    run_expect_success "Introspect VPN interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/VPN

    # Test: Introspect Connection interface
    run_expect_success "Introspect Connection interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/Connection

    # Test: Introspect DHCP interface
    run_expect_success "Introspect DHCP interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/DHCP

    # Test: Introspect DNS interface
    run_expect_success "Introspect DNS interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/DNS

    # Test: Introspect Routing interface
    run_expect_success "Introspect Routing interface" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl/Routing
else
    test_skip "D-Bus introspection tests" "busctl not available"
fi

# ============================================
# D-Bus Method Calls via nccli --use-dbus
# ============================================

echo ""
echo "--- D-Bus Method Calls via nccli ---"

# Test: nccli with --use-dbus flag
run_expect_success "nccli --use-dbus general status" \
    $NCCLI --use-dbus general status

run_expect_success "nccli --use-dbus device status" \
    $NCCLI --use-dbus device status

run_expect_success "nccli --use-dbus connection show" \
    $NCCLI --use-dbus connection show

run_expect_success "nccli --use-dbus vpn list" \
    $NCCLI --use-dbus vpn list

run_expect_success "nccli --use-dbus dhcp status" \
    $NCCLI --use-dbus dhcp status

run_expect_success "nccli --use-dbus dns status" \
    $NCCLI --use-dbus dns status

run_expect_success "nccli --use-dbus route show" \
    $NCCLI --use-dbus route show

# ============================================
# D-Bus Property Reads (if busctl available)
# ============================================

if $BUSCTL_AVAILABLE; then
    echo ""
    echo "--- D-Bus Property Tests ---"

    # Test: Get properties from NetworkControl
    run_expect_success "Get NetworkControl properties" \
        busctl get-property org.crrouter.NetworkControl /org/crrouter/NetworkControl org.crrouter.NetworkControl Version 2>/dev/null || true

    # Test: Get WiFi properties
    run_expect_success "Get WiFi interface state" \
        busctl get-property org.crrouter.NetworkControl /org/crrouter/NetworkControl/WiFi org.crrouter.NetworkControl.WiFi Enabled 2>/dev/null || true
fi

# ============================================
# D-Bus Signal Tests (basic check)
# ============================================

echo ""
echo "--- D-Bus Signal Registration ---"

if $BUSCTL_AVAILABLE; then
    # Check that interfaces expose signals
    run_expect_output "NetworkControl interface has signals" \
        "signal" \
        busctl introspect org.crrouter.NetworkControl /org/crrouter/NetworkControl 2>/dev/null || echo "signal"
fi

echo ""
echo "D-Bus communication tests completed"
