#!/bin/bash
# test_device.sh - Test device commands

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "DEVICE COMMAND TESTS"
echo "============================================"

# Test: device --help
run_expect_output "device --help shows subcommands" \
    "status" \
    $NCCLI device --help

# Test: device status
run_expect_output "device status shows DEVICE header" \
    "DEVICE" \
    $NCCLI device status

# Test: device status (terse)
run_expect_success "device status terse mode" \
    $NCCLI -t device status

# Test: device status shows loopback
run_expect_output "device status shows loopback interface" \
    "lo" \
    $NCCLI device status

# Test: device show (all devices)
run_expect_success "device show all devices" \
    $NCCLI device show

# Test: device show lo (loopback)
run_expect_output "device show lo shows GENERAL" \
    "GENERAL" \
    $NCCLI device show lo

# Test: device show nonexistent
run_expect_failure "device show nonexistent interface fails" \
    $NCCLI device show nonexistent_iface_12345

# Test: Check for ethernet interface
ETH_IFACE=$(get_ethernet_interface)
if [ -n "$ETH_IFACE" ]; then
    run_expect_output "device show ethernet interface ($ETH_IFACE)" \
        "GENERAL" \
        $NCCLI device show "$ETH_IFACE"

    run_expect_success "device status for ethernet ($ETH_IFACE)" \
        $NCCLI device status "$ETH_IFACE"
else
    test_skip "device show ethernet" "No ethernet interface found"
fi

# Test: Check for wifi interface
WIFI_IFACE=$(get_wifi_interface)
if [ -n "$WIFI_IFACE" ]; then
    run_expect_output "device show wifi interface ($WIFI_IFACE)" \
        "GENERAL" \
        $NCCLI device show "$WIFI_IFACE"

    # Test: device wifi list
    run_expect_success "device wifi list" \
        $NCCLI device wifi list

    # Test: device wifi list with interface
    run_expect_success "device wifi list on $WIFI_IFACE" \
        $NCCLI device wifi list "$WIFI_IFACE"

    # Test: device wifi --help
    run_expect_output "device wifi --help shows subcommands" \
        "list" \
        $NCCLI device wifi --help
else
    test_skip "device wifi commands" "No WiFi interface found"
fi

# Test: device set (requires root)
if check_root; then
    if [ -n "$ETH_IFACE" ]; then
        run_expect_success "device set autoconnect" \
            $NCCLI device set "$ETH_IFACE" --autoconnect yes
    fi
else
    test_skip "device set commands" "Requires root privileges"
fi

# Test: device lldp
run_expect_success "device lldp shows neighbors" \
    $NCCLI device lldp

# Test: device monitor --help (don't actually run monitor as it blocks)
run_expect_output "device monitor --help" \
    "Monitor" \
    $NCCLI device monitor --help 2>&1 || $NCCLI device --help

echo ""
echo "Device command tests completed"
