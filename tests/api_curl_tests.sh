#!/bin/bash
#
# CRRouter Web API - Comprehensive Curl Test Suite
#
# This script tests all REST API endpoints provided by crrouter-web.
# It performs both positive and negative test cases for comprehensive coverage.
#
# Usage:
#   ./api_curl_tests.sh [BASE_URL]
#
# Example:
#   ./api_curl_tests.sh http://localhost:3000
#

set -e

# Configuration
BASE_URL="${1:-http://localhost:3000}"
PASSED=0
FAILED=0
SKIPPED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED++))
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED++))
}

log_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# Test helper function
test_endpoint() {
    local test_name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_status="$5"

    log_info "Testing: $test_name"

    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "${BASE_URL}${endpoint}")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            "${BASE_URL}${endpoint}")
    fi

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" == "$expected_status" ]; then
        log_success "$test_name - Status: $http_code"
        if [ -n "$body" ]; then
            echo "  Response: $(echo "$body" | head -c 200)"
        fi
        return 0
    else
        log_error "$test_name - Expected: $expected_status, Got: $http_code"
        if [ -n "$body" ]; then
            echo "  Response: $body"
        fi
        return 1
    fi
}

# Verify server is running
log_section "Checking if server is running"
if ! curl -s --max-time 5 "${BASE_URL}/health" > /dev/null 2>&1; then
    log_error "Server is not running at ${BASE_URL}"
    log_info "Please start the server with: cargo run --bin crrouter-web"
    exit 1
fi
log_success "Server is running at ${BASE_URL}"

# ============================================================================
# Health and Info Endpoints
# ============================================================================

log_section "Health and Info Endpoints"

test_endpoint \
    "Health check" \
    "GET" \
    "/health" \
    "" \
    "200"

test_endpoint \
    "API documentation" \
    "GET" \
    "/api" \
    "" \
    "200"

# ============================================================================
# Device Management Endpoints
# ============================================================================

log_section "Device Management Endpoints"

test_endpoint \
    "List all devices" \
    "GET" \
    "/api/devices" \
    "" \
    "200"

test_endpoint \
    "List devices filtered by type (wifi)" \
    "GET" \
    "/api/devices?type=wifi" \
    "" \
    "200"

test_endpoint \
    "List devices filtered by type (ethernet)" \
    "GET" \
    "/api/devices?type=ethernet" \
    "" \
    "200"

test_endpoint \
    "List devices with invalid type filter" \
    "GET" \
    "/api/devices?type=invalid_type" \
    "" \
    "400"

# Get a real device name for testing (try common interface names)
DEVICE_NAME=""
for iface in eth0 wlan0 lo enp0s3 ens33; do
    if ip link show "$iface" > /dev/null 2>&1; then
        DEVICE_NAME="$iface"
        break
    fi
done

if [ -n "$DEVICE_NAME" ]; then
    test_endpoint \
        "Get device info for $DEVICE_NAME" \
        "GET" \
        "/api/devices/$DEVICE_NAME" \
        "" \
        "200"

    test_endpoint \
        "Get device stats for $DEVICE_NAME" \
        "GET" \
        "/api/devices/$DEVICE_NAME/stats" \
        "" \
        "200"

    # Test device configuration (should succeed or return proper error)
    test_endpoint \
        "Configure device $DEVICE_NAME (set MTU)" \
        "PATCH" \
        "/api/devices/$DEVICE_NAME" \
        '{"mtu": 1500}' \
        "200" || log_skip "Device configuration may require elevated privileges"
else
    log_skip "No suitable device found for device-specific tests"
fi

test_endpoint \
    "Get non-existent device" \
    "GET" \
    "/api/devices/nonexistent999" \
    "" \
    "404"

# Test virtual device operations (may require privileges)
log_info "Testing virtual device creation and deletion (may require root)"

VETH_NAME="veth_test_$$"
test_endpoint \
    "Delete virtual device (cleanup if exists)" \
    "DELETE" \
    "/api/devices/$VETH_NAME" \
    "" \
    "200" 2>/dev/null || log_skip "Virtual device operations require elevated privileges"

# ============================================================================
# DHCP Testing Endpoints
# ============================================================================

log_section "DHCP Testing Endpoints"

# DHCP tests require a valid interface and may require privileges
if [ -n "$DEVICE_NAME" ] && [ "$DEVICE_NAME" != "lo" ]; then
    log_info "Using interface $DEVICE_NAME for DHCP tests"

    # Note: These tests may fail if not run with appropriate privileges or
    # if the interface is not in a state that allows DHCP testing

    test_endpoint \
        "DHCP discover on $DEVICE_NAME" \
        "POST" \
        "/api/dhcp/discover" \
        "{\"interface\": \"$DEVICE_NAME\"}" \
        "200" || log_skip "DHCP discover may require elevated privileges or proper network setup"

    test_endpoint \
        "DHCP test with discover message type" \
        "POST" \
        "/api/dhcp/test" \
        "{\"interface\": \"$DEVICE_NAME\", \"message_type\": \"Discover\"}" \
        "200" || log_skip "DHCP test may require elevated privileges"

    # DHCP request requires additional parameters
    test_endpoint \
        "DHCP request on $DEVICE_NAME" \
        "POST" \
        "/api/dhcp/request" \
        "{\"interface\": \"$DEVICE_NAME\", \"message_type\": \"Request\", \"requested_ip\": \"192.168.1.100\", \"server_ip\": \"192.168.1.1\"}" \
        "200" || log_skip "DHCP request may require elevated privileges or specific network setup"

    # DHCP release requires an existing lease
    test_endpoint \
        "DHCP release on $DEVICE_NAME" \
        "POST" \
        "/api/dhcp/release" \
        "{\"interface\": \"$DEVICE_NAME\", \"message_type\": \"Release\", \"client_ip\": \"192.168.1.100\", \"server_ip\": \"192.168.1.1\"}" \
        "200" || log_skip "DHCP release may require elevated privileges or active lease"

    test_endpoint \
        "DHCP test sequence on $DEVICE_NAME" \
        "GET" \
        "/api/dhcp/test-sequence/$DEVICE_NAME" \
        "" \
        "200" || log_skip "DHCP test sequence may require elevated privileges"
else
    log_skip "No suitable interface for DHCP testing (found: ${DEVICE_NAME:-none})"
fi

# Test DHCP with invalid interface
test_endpoint \
    "DHCP discover on non-existent interface" \
    "POST" \
    "/api/dhcp/discover" \
    '{"interface": "nonexistent999"}' \
    "404" || test_endpoint \
    "DHCP discover on non-existent interface (may return 500)" \
    "POST" \
    "/api/dhcp/discover" \
    '{"interface": "nonexistent999"}' \
    "500"

# ============================================================================
# Interface Management Endpoints (Legacy)
# ============================================================================

log_section "Interface Management Endpoints (Legacy)"

test_endpoint \
    "List all network interfaces" \
    "GET" \
    "/api/interfaces" \
    "" \
    "200"

if [ -n "$DEVICE_NAME" ]; then
    test_endpoint \
        "Get interface info for $DEVICE_NAME" \
        "GET" \
        "/api/interfaces/$DEVICE_NAME" \
        "" \
        "200"
fi

test_endpoint \
    "Get non-existent interface" \
    "GET" \
    "/api/interfaces/nonexistent999" \
    "" \
    "404"

# ============================================================================
# WiFi Endpoints
# ============================================================================

log_section "WiFi Endpoints"

# Find a WiFi interface for testing
WIFI_IFACE=""
for iface in wlan0 wlp2s0 wlp3s0; do
    if ip link show "$iface" > /dev/null 2>&1; then
        WIFI_IFACE="$iface"
        break
    fi
done

if [ -n "$WIFI_IFACE" ]; then
    test_endpoint \
        "WiFi scan on $WIFI_IFACE" \
        "GET" \
        "/api/wifi/scan/$WIFI_IFACE" \
        "" \
        "200" || log_skip "WiFi scan may require elevated privileges or wireless interface"
else
    log_skip "No WiFi interface found for WiFi scan tests"
fi

# Test WiFi scan with non-existent interface
test_endpoint \
    "WiFi scan on non-existent interface" \
    "GET" \
    "/api/wifi/scan/nonexistent999" \
    "" \
    "404" || test_endpoint \
    "WiFi scan on non-existent interface (may return 500)" \
    "GET" \
    "/api/wifi/scan/nonexistent999" \
    "" \
    "500"

# ============================================================================
# Error Handling Tests
# ============================================================================

log_section "Error Handling Tests"

test_endpoint \
    "Invalid endpoint (404)" \
    "GET" \
    "/api/invalid/endpoint" \
    "" \
    "404"

test_endpoint \
    "Malformed JSON in POST request" \
    "POST" \
    "/api/dhcp/discover" \
    '{invalid json}' \
    "400" || test_endpoint \
    "Malformed JSON in POST request (may return 422)" \
    "POST" \
    "/api/dhcp/discover" \
    '{invalid json}' \
    "422"

# ============================================================================
# Test Summary
# ============================================================================

log_section "Test Summary"

TOTAL=$((PASSED + FAILED + SKIPPED))

echo ""
echo "Total tests: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
