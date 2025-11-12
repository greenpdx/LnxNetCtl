# Building the Debian Package

This document explains how to build the netctl Debian package.

## Prerequisites

Install the required build dependencies:

```bash
sudo apt install debhelper devscripts build-essential
sudo apt install cargo rustc pkg-config libc6-dev
```

For Rust, ensure you have at least version 1.70:
```bash
rustc --version
```

If you need to update Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building the Package

### Method 1: Using dpkg-buildpackage (Recommended)

From the project root directory:

```bash
# Clean any previous builds
cargo clean

# Build the package
dpkg-buildpackage -us -uc -b

# The .deb file will be created in the parent directory
ls -lh ../netctl_*.deb
```

Options:
- `-us`: Do not sign the source package
- `-uc`: Do not sign the changes file
- `-b`: Binary-only build (no source package)

### Method 2: Using debuild

```bash
# Install debuild if not already installed
sudo apt install devscripts

# Build with debuild
debuild -us -uc -b

# Package will be in parent directory
ls -lh ../netctl_*.deb
```

### Method 3: Manual build with dpkg-buildpackage

```bash
# Clean build directory
fakeroot debian/rules clean

# Build binaries
debian/rules build

# Create package
fakeroot debian/rules binary

# Package in parent directory
ls -lh ../netctl_*.deb
```

## Installing the Package

After building:

```bash
sudo dpkg -i ../netctl_*.deb
```

If there are dependency issues:

```bash
sudo apt --fix-broken install
```

## Verifying the Package

Check package contents:

```bash
dpkg -c ../netctl_*.deb
```

Check package information:

```bash
dpkg -I ../netctl_*.deb
```

## Package Contents

The package includes:

**Binaries:**
- `/usr/bin/netctl` - Main network control tool
- `/usr/bin/nm-converter` - NetworkManager config converter

**Man Pages:**
- `/usr/share/man/man1/netctl.1.gz`
- `/usr/share/man/man1/nm-converter.1.gz`
- `/usr/share/man/man5/netctl.nctl.5.gz`
- `/usr/share/man/man7/netctl-plugin.7.gz`

**Systemd Services:**
- `/lib/systemd/system/netctl.service`
- `/lib/systemd/system/netctl@.service`
- `/lib/systemd/system/netctl-auto@.service`

**Configuration:**
- `/etc/netctl/` - Configuration directory
- `/etc/netctl/plugins/` - Plugin configuration

**Documentation:**
- `/usr/share/doc/netctl/README.md`
- `/usr/share/doc/netctl/README.Debian`
- `/usr/share/doc/netctl/examples/` - Example configurations
- `/usr/share/doc/netctl/copyright`
- `/usr/share/doc/netctl/changelog.Debian.gz`

## Troubleshooting Build Issues

### Cargo/Rust Issues

If cargo fails to build:

```bash
# Update Rust
rustup update

# Clean cargo cache
cargo clean
rm -rf ~/.cargo/registry
```

### Missing Dependencies

If build fails due to missing dependencies:

```bash
# Install build dependencies from debian/control
sudo apt build-dep .
```

### Permission Issues

If you get permission errors:

```bash
# Clean with sudo
sudo cargo clean
sudo debian/rules clean

# Fix ownership
sudo chown -R $USER:$USER .

# Try building again
dpkg-buildpackage -us -uc -b
```

## Cross-Compilation

To build for different architectures:

```bash
# Install cross-compilation tools
sudo apt install crossbuild-essential-arm64

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Configure cargo for cross-compilation
# Edit ~/.cargo/config.toml:
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

# Build for ARM64
dpkg-buildpackage -aarm64 -us -uc -b
```

## Building Source Package

To create a source package for upload to repositories:

```bash
# Build source package
dpkg-buildpackage -S -us -uc

# This creates:
# - netctl_*.dsc
# - netctl_*.tar.xz
# - netctl_*.changes
```

## Signing Packages

For official releases, sign the packages:

```bash
# Build and sign
dpkg-buildpackage -sa

# Or sign after building
debsign ../netctl_*.changes
```

## Creating a Repository

To create a local APT repository:

```bash
# Install reprepro
sudo apt install reprepro

# Create repository structure
mkdir -p ~/apt-repo/conf

# Configure repository
cat > ~/apt-repo/conf/distributions <<EOF
Origin: netctl
Label: netctl
Codename: stable
Architectures: amd64 arm64 armhf
Components: main
Description: netctl network management tool
SignWith: your-gpg-key-id
EOF

# Add package to repository
reprepro -b ~/apt-repo includedeb stable ../netctl_*.deb

# Use repository
# Add to /etc/apt/sources.list:
# deb [trusted=yes] file:///home/user/apt-repo stable main
```

## CI/CD Integration

For automated builds, use GitHub Actions or GitLab CI:

```yaml
# .github/workflows/build-deb.yml
name: Build Debian Package

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: |
          sudo apt update
          sudo apt install -y debhelper devscripts cargo rustc
      - name: Build package
        run: dpkg-buildpackage -us -uc -b
      - name: Upload artifact
        uses: actions/upload-artifact@v2
        with:
          name: debian-package
          path: ../*.deb
```

## Support

For build issues, report at:
https://github.com/netctl/netctl/issues
