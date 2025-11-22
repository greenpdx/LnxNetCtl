# Makefile for netctl documentation
# Builds man pages and GNU info documentation

# Installation directories
PREFIX ?= /usr
MANDIR ?= $(PREFIX)/share/man
INFODIR ?= $(PREFIX)/share/info
DOCDIR ?= $(PREFIX)/share/doc/netctl

# Man page sections
MAN1DIR = $(MANDIR)/man1
MAN5DIR = $(MANDIR)/man5
MAN7DIR = $(MANDIR)/man7

# Source files
TEXI_SRC = netctl.texi
INFO_OUT = netctl.info

# Man pages
MAN1_PAGES = netctl.1 nm-converter.1 nccli.1
MAN5_PAGES = netctl.nctl.5
MAN7_PAGES = netctl-plugin.7

# Tools
MAKEINFO = makeinfo
INSTALL = install
INSTALL_DATA = $(INSTALL) -m 644
GZIP = gzip

.PHONY: all info man clean install install-info install-man uninstall help

# Default target
all: info

# Build info documentation from Texinfo source
info: $(INFO_OUT)

$(INFO_OUT): $(TEXI_SRC)
	@echo "Building info documentation..."
	$(MAKEINFO) --no-split -o $@ $<
	@echo "Info documentation built successfully: $@"

# Man pages are already in roff format, no build needed
man:
	@echo "Man pages are ready (no build required)"
	@echo "Man pages: $(MAN1_PAGES) $(MAN5_PAGES) $(MAN7_PAGES)"

# Install all documentation
install: install-info install-man
	@echo "All documentation installed successfully"

# Install info documentation
install-info: info
	@echo "Installing info documentation..."
	$(INSTALL) -d $(DESTDIR)$(INFODIR)
	$(INSTALL_DATA) $(INFO_OUT) $(DESTDIR)$(INFODIR)/
	@if command -v install-info >/dev/null 2>&1; then \
		install-info $(DESTDIR)$(INFODIR)/$(INFO_OUT) $(DESTDIR)$(INFODIR)/dir 2>/dev/null || true; \
		echo "Info documentation registered in dir file"; \
	else \
		echo "install-info not found, skipping dir file update"; \
	fi
	@echo "Info documentation installed to $(INFODIR)"

# Install man pages
install-man:
	@echo "Installing man pages..."
	$(INSTALL) -d $(DESTDIR)$(MAN1DIR)
	$(INSTALL) -d $(DESTDIR)$(MAN5DIR)
	$(INSTALL) -d $(DESTDIR)$(MAN7DIR)
	@for man in $(MAN1_PAGES); do \
		echo "  Installing $$man to $(MAN1DIR)/"; \
		$(INSTALL_DATA) $$man $(DESTDIR)$(MAN1DIR)/; \
	done
	@for man in $(MAN5_PAGES); do \
		echo "  Installing $$man to $(MAN5DIR)/"; \
		$(INSTALL_DATA) $$man $(DESTDIR)$(MAN5DIR)/; \
	done
	@for man in $(MAN7_PAGES); do \
		echo "  Installing $$man to $(MAN7DIR)/"; \
		$(INSTALL_DATA) $$man $(DESTDIR)$(MAN7DIR)/; \
	done
	@if command -v mandb >/dev/null 2>&1; then \
		echo "Updating man database..."; \
		mandb -q 2>/dev/null || true; \
	fi
	@echo "Man pages installed successfully"

# Uninstall documentation
uninstall: uninstall-info uninstall-man
	@echo "All documentation uninstalled successfully"

# Uninstall info documentation
uninstall-info:
	@echo "Uninstalling info documentation..."
	@if [ -f $(DESTDIR)$(INFODIR)/$(INFO_OUT) ]; then \
		if command -v install-info >/dev/null 2>&1; then \
			install-info --delete $(DESTDIR)$(INFODIR)/$(INFO_OUT) $(DESTDIR)$(INFODIR)/dir 2>/dev/null || true; \
		fi; \
		rm -f $(DESTDIR)$(INFODIR)/$(INFO_OUT); \
		echo "Info documentation removed from $(INFODIR)"; \
	else \
		echo "Info documentation not found, nothing to remove"; \
	fi

# Uninstall man pages
uninstall-man:
	@echo "Uninstalling man pages..."
	@for man in $(MAN1_PAGES); do \
		if [ -f $(DESTDIR)$(MAN1DIR)/$$man ]; then \
			echo "  Removing $(MAN1DIR)/$$man"; \
			rm -f $(DESTDIR)$(MAN1DIR)/$$man; \
		fi; \
	done
	@for man in $(MAN5_PAGES); do \
		if [ -f $(DESTDIR)$(MAN5DIR)/$$man ]; then \
			echo "  Removing $(MAN5DIR)/$$man"; \
			rm -f $(DESTDIR)$(MAN5DIR)/$$man; \
		fi; \
	done
	@for man in $(MAN7_PAGES); do \
		if [ -f $(DESTDIR)$(MAN7DIR)/$$man ]; then \
			echo "  Removing $(MAN7DIR)/$$man"; \
			rm -f $(DESTDIR)$(MAN7DIR)/$$man; \
		fi; \
	done
	@if command -v mandb >/dev/null 2>&1; then \
		echo "Updating man database..."; \
		mandb -q 2>/dev/null || true; \
	fi

# Clean generated files
clean:
	@echo "Cleaning generated files..."
	rm -f $(INFO_OUT)
	rm -f *.aux *.cp *.fn *.ky *.log *.pg *.toc *.tp *.vr
	@echo "Clean complete"

# Validate info documentation
check: info
	@echo "Validating info documentation..."
	@if command -v info >/dev/null 2>&1; then \
		info --file=$(INFO_OUT) --node=Top --output=/dev/null && \
		echo "Info documentation is valid"; \
	else \
		echo "info command not found, skipping validation"; \
	fi

# Show help
help:
	@echo "netctl documentation Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build all documentation (default: info)"
	@echo "  info         - Build GNU info documentation from Texinfo source"
	@echo "  man          - Check man pages (no build needed)"
	@echo "  install      - Install all documentation"
	@echo "  install-info - Install info documentation"
	@echo "  install-man  - Install man pages"
	@echo "  uninstall    - Uninstall all documentation"
	@echo "  clean        - Remove generated files"
	@echo "  check        - Validate info documentation"
	@echo "  help         - Show this help message"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX       - Installation prefix (default: /usr)"
	@echo "  MANDIR       - Man page directory (default: \$${PREFIX}/share/man)"
	@echo "  INFODIR      - Info directory (default: \$${PREFIX}/share/info)"
	@echo "  DESTDIR      - Staging directory for package builds"
	@echo ""
	@echo "Examples:"
	@echo "  make info                    - Build info documentation"
	@echo "  sudo make install            - Install all documentation"
	@echo "  sudo make install-man        - Install only man pages"
	@echo "  make PREFIX=/usr/local       - Use different prefix"
	@echo "  make DESTDIR=/tmp/pkg install - Stage installation"
