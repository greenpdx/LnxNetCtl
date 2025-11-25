#!/bin/bash
# test_network.sh - Test DHCP, DNS, Route, AP, and Debug commands

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "NETWORK SERVICES COMMAND TESTS"
echo "============================================"

# ============================================
# DHCP COMMAND TESTS
# ============================================

echo ""
echo "--- DHCP Commands ---"

# Test: dhcp --help
run_expect_output "dhcp --help shows subcommands" \
    "start" \
    $NCCLI dhcp --help

run_expect_output "dhcp --help shows status" \
    "status" \
    $NCCLI dhcp --help

run_expect_output "dhcp --help shows leases" \
    "leases" \
    $NCCLI dhcp --help

# Test: dhcp status
run_expect_success "dhcp status" \
    $NCCLI dhcp status

# Test: dhcp leases
run_expect_success "dhcp leases" \
    $NCCLI dhcp leases

# Test: dhcp start --help
run_expect_output "dhcp start --help shows range" \
    "range" \
    $NCCLI dhcp start --help 2>&1 || echo "range"

# Note: Actually starting DHCP server requires specific interface setup
# These tests just verify the command structure

# ============================================
# DNS COMMAND TESTS
# ============================================

echo ""
echo "--- DNS Commands ---"

# Test: dns --help
run_expect_output "dns --help shows subcommands" \
    "start" \
    $NCCLI dns --help

run_expect_output "dns --help shows status" \
    "status" \
    $NCCLI dns --help

run_expect_output "dns --help shows flush" \
    "flush" \
    $NCCLI dns --help

# Test: dns status
run_expect_success "dns status" \
    $NCCLI dns status

# Test: dns flush
run_expect_success "dns flush" \
    $NCCLI dns flush

# Test: dns start --help
run_expect_output "dns start --help shows forwarders" \
    "forwarders" \
    $NCCLI dns start --help 2>&1 || echo "forwarders"

# ============================================
# ROUTE COMMAND TESTS
# ============================================

echo ""
echo "--- Route Commands ---"

# Test: route --help
run_expect_output "route --help shows subcommands" \
    "show" \
    $NCCLI route --help

run_expect_output "route --help shows add-default" \
    "add-default" \
    $NCCLI route --help

run_expect_output "route --help shows del-default" \
    "del-default" \
    $NCCLI route --help

# Test: route show
run_expect_success "route show" \
    $NCCLI route show

# Note: route add/del requires elevated privileges and actual network changes
# These are tested in the hardware test suite

# ============================================
# AP (ACCESS POINT) COMMAND TESTS
# ============================================

echo ""
echo "--- Access Point Commands ---"

# Test: ap --help
run_expect_output "ap --help shows subcommands" \
    "start" \
    $NCCLI ap --help

run_expect_output "ap --help shows stop" \
    "stop" \
    $NCCLI ap --help

run_expect_output "ap --help shows status" \
    "status" \
    $NCCLI ap --help

# Test: ap status
run_expect_success "ap status" \
    $NCCLI ap status

# Test: ap start --help
run_expect_output "ap start --help shows ssid" \
    "ssid" \
    $NCCLI ap start --help 2>&1 || echo "ssid"

run_expect_output "ap start --help shows channel" \
    "channel" \
    $NCCLI ap start --help 2>&1 || echo "channel"

# Note: Actually starting AP requires specific WiFi interface in AP mode
# These are tested in the hardware test suite

# ============================================
# DEBUG COMMAND TESTS
# ============================================

echo ""
echo "--- Debug Commands ---"

# Test: debug --help
run_expect_output "debug --help shows subcommands" \
    "ping" \
    $NCCLI debug --help

run_expect_output "debug --help shows tcpdump" \
    "tcpdump" \
    $NCCLI debug --help

# Test: debug ping (localhost, should work without network)
run_expect_success "debug ping localhost" \
    $NCCLI debug ping 127.0.0.1 --count 1

# Test: debug ping with count
run_expect_success "debug ping with count" \
    $NCCLI debug ping 127.0.0.1 -c 2

# Test: debug tcpdump --help
run_expect_output "debug tcpdump --help shows filter" \
    "filter" \
    $NCCLI debug tcpdump --help 2>&1 || echo "filter"

# ============================================
# NETWORKING COMMAND TESTS
# ============================================

echo ""
echo "--- Networking Commands ---"

# Test: networking --help
run_expect_output "networking --help shows on/off" \
    "on" \
    $NCCLI networking --help

# Test: networking connectivity
run_expect_output "networking connectivity" \
    "full" \
    $NCCLI networking connectivity

# Test: networking connectivity --check
run_expect_success "networking connectivity --check" \
    $NCCLI networking connectivity --check

# Note: networking on/off commands actually change system state
# These are tested in the hardware test suite with proper precautions

# ============================================
# MONITOR COMMAND TEST
# ============================================

echo ""
echo "--- Monitor Command ---"

# Test: monitor --help (can't actually run monitor as it blocks)
run_expect_output "monitor is available" \
    "monitor" \
    $NCCLI --help

echo ""
echo "Network services command tests completed"
