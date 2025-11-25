# nccli Integration Tests

This directory contains shell-based integration tests for the `nccli` command-line tool.
These tests require the `netctld` daemon to be running.

## Prerequisites

1. Build the project: `cargo build`
2. Start the daemon: `sudo target/debug/netctld`
3. Run tests in another terminal

## Running Tests

```bash
# Run all tests
sudo ./test/nccli/run_tests.sh

# Run specific test category
sudo ./test/nccli/run_tests.sh general
sudo ./test/nccli/run_tests.sh device
sudo ./test/nccli/run_tests.sh connection
sudo ./test/nccli/run_tests.sh vpn
sudo ./test/nccli/run_tests.sh network
```

## Test Structure

- `run_tests.sh` - Main test runner
- `test_general.sh` - General command tests
- `test_device.sh` - Device management tests
- `test_connection.sh` - Connection management tests
- `test_vpn.sh` - VPN command tests
- `test_network.sh` - DHCP, DNS, Route tests
- `test_helpers.sh` - Common test utilities

## Test Categories

### General Commands
- `nccli general status`
- `nccli general hostname`
- `nccli general permissions`
- `nccli general logging`

### Device Commands
- `nccli device status`
- `nccli device show`
- `nccli device wifi list`

### Connection Commands
- `nccli connection show`
- `nccli connection add`
- `nccli connection modify`
- `nccli connection delete`

### VPN Commands
- `nccli vpn list`
- `nccli vpn backends`
- `nccli vpn status`

### Network Commands
- `nccli dhcp status`
- `nccli dns status`
- `nccli route show`
