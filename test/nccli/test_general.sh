#!/bin/bash
# test_general.sh - Test general commands

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helpers.sh"

echo "============================================"
echo "GENERAL COMMAND TESTS"
echo "============================================"

# Test: nccli --help
run_expect_output "nccli --help shows usage" \
    "Network Control CLI" \
    $NCCLI --help

# Test: nccli --version
run_expect_output "nccli --version shows version" \
    "nccli" \
    $NCCLI --version

# Test: nccli (default status)
run_expect_success "nccli default command runs" \
    $NCCLI

# Test: nccli general status
run_expect_output "general status shows STATE" \
    "STATE" \
    $NCCLI general status

# Test: nccli general status (terse)
run_expect_output "general status terse mode" \
    "running" \
    $NCCLI -t general status

# Test: nccli general permissions
run_expect_output "general permissions shows capabilities" \
    "network.control" \
    $NCCLI general permissions

# Test: nccli general permissions (terse)
run_expect_output "general permissions terse mode" \
    "network.control:yes" \
    $NCCLI -t general permissions

# Test: nccli general logging
run_expect_output "general logging shows LEVEL" \
    "LEVEL" \
    $NCCLI general logging

# Test: nccli general hostname
run_expect_success "general hostname shows hostname" \
    $NCCLI general hostname

# Test: nccli general --help
run_expect_output "general --help shows subcommands" \
    "status" \
    $NCCLI general --help

# Test: output modes
run_expect_success "tabular output mode" \
    $NCCLI -m tabular general status

run_expect_success "multiline output mode" \
    $NCCLI -m multiline general status

run_expect_success "terse output mode" \
    $NCCLI -m terse general status

# Test: networking commands
run_expect_output "networking connectivity shows state" \
    "full" \
    $NCCLI networking connectivity

# Test: radio commands
run_expect_output "radio all shows WIFI" \
    "WIFI" \
    $NCCLI radio all

run_expect_output "radio all terse mode" \
    "enabled" \
    $NCCLI -t radio all

echo ""
echo "General command tests completed"
