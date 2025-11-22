# netctl Documentation

This directory contains all documentation for netctl, including man pages and GNU info documentation.

## Documentation Files

### Man Pages

Man pages are provided in standard roff format:

- **netctl.1** - Main netctl command reference (section 1)
- **nm-converter.1** - NetworkManager configuration converter (section 1)
- **libnccli.1** - Comprehensive network CLI tool (section 1)
- **netctl.nctl.5** - Connection configuration file format (section 5)
- **netctl-plugin.7** - Plugin development guide (section 7)

### Info Documentation

- **netctl.texi** - Texinfo source for comprehensive GNU info documentation
- **netctl.info** - Generated info file (built from netctl.texi)

### Additional Documentation

- **CONNECTION_MANAGEMENT.md** - Connection management guide
- **CR_DBUS_API.md** - D-Bus API documentation
- **DBUS_TEST_GUIDE.md** - D-Bus testing guide
- **DHCP_CLIENT_INTEGRATION.md** - DHCP client integration guide
- **LIBCR_COMPAT_API.md** - Library compatibility API
- **libnccli.md** - libnccli detailed documentation

## Building Documentation

### Prerequisites

To build the info documentation, you need the texinfo package:

```bash
# Debian/Ubuntu
sudo apt install texinfo

# Fedora/RHEL
sudo dnf install texinfo

# Arch Linux
sudo pacman -S texinfo
```

### Build Commands

Build info documentation:
```bash
cd docs
make info
```

Check that man pages are ready (no build needed):
```bash
make man
```

Build all documentation:
```bash
make all
```

Validate info documentation:
```bash
make check
```

Clean generated files:
```bash
make clean
```

## Installing Documentation

### Install All Documentation

```bash
cd docs
sudo make install
```

### Install Only Man Pages

```bash
sudo make install-man
```

### Install Only Info Documentation

```bash
sudo make install-info
```

### Custom Installation Prefix

```bash
# Install to /usr/local instead of /usr
sudo make install PREFIX=/usr/local

# Stage installation for package building
make install DESTDIR=/tmp/staging PREFIX=/usr
```

## Viewing Documentation

### Man Pages

View man pages directly from this directory without installing:

```bash
man -l netctl.1
man -l libnccli.1
man -l nm-converter.1
man -l netctl.nctl.5
man -l netctl-plugin.7
```

Or after installation:

```bash
man netctl
man libnccli
man nm-converter
man netctl.nctl
man netctl-plugin
```

### Info Documentation

View info documentation directly:

```bash
info -f netctl.info
```

Or after installation:

```bash
info netctl
```

Navigate info documentation:
- `Space` - Next page
- `Backspace` - Previous page
- `n` - Next node
- `p` - Previous node
- `u` - Up to parent node
- `m` - Go to menu item
- `q` - Quit
- `?` - Help

### HTML Documentation

Generate HTML from info documentation:

```bash
makeinfo --html netctl.texi
```

This creates an HTML version in the `netctl/` directory.

Generate a single HTML file:

```bash
makeinfo --html --no-split netctl.texi -o netctl.html
```

### PDF Documentation

Generate PDF from info documentation (requires texi2pdf):

```bash
texi2pdf netctl.texi
```

## Uninstalling Documentation

Remove all installed documentation:

```bash
cd docs
sudo make uninstall
```

Remove only man pages:

```bash
sudo make uninstall-man
```

Remove only info documentation:

```bash
sudo make uninstall-info
```

## Documentation Structure

The info documentation is organized hierarchically:

- **Introduction** - Overview and features
- **Installation** - Installation methods and dependencies
- **Getting Started** - Quick start guide
- **Commands** - Complete command reference
- **Connection Management** - Managing network connections
- **Device Management** - Managing network devices
- **WiFi Operations** - WiFi scanning and connection
- **Access Point Mode** - Creating WiFi hotspots
- **DHCP Configuration** - DHCP server setup
- **VPN Support** - VPN configuration
- **Configuration Files** - Connection file format
- **Tools** - Additional utilities (libnccli, nm-converter)
- **D-Bus Interface** - NetworkManager compatibility
- **Security** - Security considerations
- **Troubleshooting** - Common issues and solutions

## Contributing to Documentation

When adding new features or commands:

1. Update the relevant man page(s)
2. Update the info documentation in netctl.texi
3. Add examples where appropriate
4. Update this README if needed
5. Test the documentation:
   ```bash
   make clean
   make info
   make check
   man -l <manpage>
   info -f netctl.info
   ```

## Man Page Sections

- **Section 1**: User commands (netctl, libnccli, nm-converter)
- **Section 5**: File formats (netctl.nctl)
- **Section 7**: Miscellaneous (netctl-plugin)
- **Section 8**: System administration commands (not used currently)

## Documentation Format Standards

### Man Pages

Man pages use the standard roff format with these macros:
- `.TH` - Title header
- `.SH` - Section header
- `.SS` - Subsection header
- `.TP` - Tagged paragraph
- `.IP` - Indented paragraph
- `.EX/.EE` - Example block
- `.BR` - Bold/Roman cross-reference

### Info Documentation

Info documentation uses Texinfo format with these features:
- `@node` - Document nodes
- `@chapter`, `@section`, `@subsection` - Hierarchical structure
- `@table`, `@itemize`, `@enumerate` - Lists
- `@example` - Example blocks
- `@xref` - Cross-references
- `@cindex` - Index entries

## License

All documentation is licensed under MIT OR Apache-2.0, the same as the netctl project.
