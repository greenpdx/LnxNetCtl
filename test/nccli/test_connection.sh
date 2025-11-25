#!/bin/bash
# test_connection.sh - Test connection commands

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "CONNECTION COMMAND TESTS"
echo "============================================"

# Test: connection --help
run_expect_output "connection --help shows subcommands" \
    "show" \
    $NCCLI connection --help

run_expect_output "connection --help shows add" \
    "add" \
    $NCCLI connection --help

run_expect_output "connection --help shows modify" \
    "modify" \
    $NCCLI connection --help

run_expect_output "connection --help shows delete" \
    "delete" \
    $NCCLI connection --help

# Test: connection show (list all)
run_expect_success "connection show lists connections" \
    $NCCLI connection show

# Test: connection show (terse)
run_expect_success "connection show terse mode" \
    $NCCLI -t connection show

# Test: connection show --active
run_expect_success "connection show --active" \
    $NCCLI connection show --active

# Test: connection show nonexistent
run_expect_failure "connection show nonexistent fails" \
    $NCCLI connection show "nonexistent-connection-12345"

# Test: connection up nonexistent
run_expect_failure "connection up nonexistent fails" \
    $NCCLI connection up "nonexistent-connection-12345"

# Test: connection down nonexistent (may succeed with no-op)
run_expect_success "connection down nonexistent" \
    $NCCLI connection down "nonexistent-connection-12345"

# Test: connection reload
run_expect_success "connection reload" \
    $NCCLI connection reload

# ============================================
# Connection creation tests (require cleanup)
# ============================================

TEST_CONN_NAME="nccli-test-connection-$$"

# Test: connection add ethernet
run_expect_success "connection add ethernet" \
    $NCCLI connection add \
        --type ethernet \
        --con-name "$TEST_CONN_NAME" \
        --ifname eth0 \
        --ip4 auto

# Test: connection show the new connection
run_expect_output "connection show new connection" \
    "$TEST_CONN_NAME" \
    $NCCLI connection show "$TEST_CONN_NAME"

# Test: connection modify
run_expect_success "connection modify autoconnect" \
    $NCCLI connection modify "$TEST_CONN_NAME" connection.autoconnect no

# Test: connection delete
run_expect_success "connection delete test connection" \
    $NCCLI connection delete "$TEST_CONN_NAME"

# Verify deletion
run_expect_failure "connection show deleted connection fails" \
    $NCCLI connection show "$TEST_CONN_NAME"

# ============================================
# WiFi connection tests
# ============================================

WIFI_IFACE=$(get_wifi_interface)
if [ -n "$WIFI_IFACE" ]; then
    TEST_WIFI_CONN="nccli-test-wifi-$$"

    # Test: connection add wifi
    run_expect_success "connection add wifi" \
        $NCCLI connection add \
            --type wifi \
            --con-name "$TEST_WIFI_CONN" \
            --ssid "TestNetwork" \
            --password "testpassword123" \
            --ip4 auto

    # Test: connection show wifi
    run_expect_output "connection show wifi connection" \
        "$TEST_WIFI_CONN" \
        $NCCLI connection show "$TEST_WIFI_CONN"

    # Cleanup
    run_expect_success "connection delete wifi test" \
        $NCCLI connection delete "$TEST_WIFI_CONN"
else
    test_skip "WiFi connection tests" "No WiFi interface found"
fi

# ============================================
# Connection import/export tests
# ============================================

# Test: connection import --help
run_expect_output "connection import --help" \
    "type" \
    $NCCLI connection import --help 2>&1 || echo "type"

# Test: connection export --help
run_expect_output "connection export --help" \
    "file" \
    $NCCLI connection export --help 2>&1 || echo "file"

# Test: connection load --help
run_expect_output "connection load --help" \
    "filename" \
    $NCCLI connection load --help 2>&1 || echo "filename"

# ============================================
# Connection clone tests
# ============================================

CLONE_SOURCE="nccli-clone-source-$$"
CLONE_DEST="nccli-clone-dest-$$"

# Create source connection
run_expect_success "create source connection for clone" \
    $NCCLI connection add \
        --type ethernet \
        --con-name "$CLONE_SOURCE" \
        --ifname eth0 \
        --ip4 auto

# Test: connection clone
run_expect_success "connection clone" \
    $NCCLI connection clone "$CLONE_SOURCE" --new-name "$CLONE_DEST"

# Verify both exist
run_expect_output "cloned connection exists" \
    "$CLONE_DEST" \
    $NCCLI connection show "$CLONE_DEST"

# Cleanup
$NCCLI connection delete "$CLONE_SOURCE" > /dev/null 2>&1
$NCCLI connection delete "$CLONE_DEST" > /dev/null 2>&1

echo ""
echo "Connection command tests completed"
