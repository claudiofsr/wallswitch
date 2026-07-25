# ==============================================================================
# SECURE MAKEFILE FOR WALLSWITCH
# Designed for maximum portability, safety, and defensive execution across
# all Linux distributions (Ubuntu, Arch/Manjaro, Alpine, Void, etc.)
# ==============================================================================

# --- 1. Bulletproof Shell Selection ---
# We try to find bash dynamically. If found, we use it with strict safety flags.
# If not (e.g., Alpine Linux), we fallback to strict POSIX sh.
BASH_PATH := $(shell command -v bash 2>/dev/null)

ifneq ($(BASH_PATH),)
    SHELL := $(BASH_PATH)
    # -e: Exit immediately on error
    # -u: Treat unset variables as errors
    # -o pipefail: Catch errors hidden inside pipelines (bash/zsh feature)
    .SHELLFLAGS := -euo pipefail -c
else
    # Fallback for minimal distributions without bash (Dash, Ash, Busybox)
    SHELL := /bin/sh
    .SHELLFLAGS := -euc
endif

# --- 2. Safe Environment Defaults ---
INTERVAL ?= 600

# Safety Check: Prevent disastrous 'rm' commands if HOME is somehow unset
ifeq ($(HOME),)
    $(error "CRITICAL ERROR: $$HOME environment variable is not set. Aborting to prevent unsafe filesystem operations.")
endif

# --- 3. Dynamic Dependency Resolution ---
CARGO     := $(shell command -v cargo 2>/dev/null)
PYTHON3   := $(shell command -v python3 2>/dev/null)
SYSTEMCTL := $(shell command -v systemctl 2>/dev/null)

.PHONY: all build install setup-systemd uninstall check-requirements

all: build

# --- 4. Strict Requirement Validation ---
check-requirements:
ifeq ($(CARGO),)
	$(error "CRITICAL: 'cargo' binary not found in PATH. Please install Rust (rustup).")
endif
ifeq ($(PYTHON3),)
	$(error "CRITICAL: 'python3' binary not found in PATH. Please install Python 3.")
endif
	@if [ ! -f "setup_systemd.py" ]; then \
		echo "CRITICAL: 'setup_systemd.py' is missing in the current directory." >&2; \
		exit 1; \
	fi

build: check-requirements
	"$(CARGO)" build --release

install: check-requirements
	"$(CARGO)" install --path=.
	"$(PYTHON3)" setup_systemd.py --seconds "$(INTERVAL)"

setup-systemd: check-requirements
	"$(PYTHON3)" setup_systemd.py --seconds "$(INTERVAL)"

# --- 5. Safe and Graceful Uninstallation ---
uninstall:
	@echo "=> Starting uninstallation process..."
	@echo "=> Checking systemd configuration..."
	@if [ -n "$(SYSTEMCTL)" ]; then \
		"$(SYSTEMCTL)" --user disable --now wallswitch.timer 2>/dev/null || true; \
	else \
		echo "   (systemctl not found; skipping systemd timer deactivation)"; \
	fi
	
	@echo "=> Removing generated unit files safely..."
	-rm -f "$(HOME)/.config/systemd/user/wallswitch.service"
	-rm -f "$(HOME)/.config/systemd/user/wallswitch.timer"
	
	@if [ -n "$(SYSTEMCTL)" ]; then \
		"$(SYSTEMCTL)" --user daemon-reload 2>/dev/null || true; \
	fi
	
	@echo "=> Uninstalling cargo binary..."
	@if [ -n "$(CARGO)" ]; then \
		"$(CARGO)" uninstall wallswitch 2>/dev/null || true; \
	else \
		echo "   (cargo not found; cannot cleanly uninstall binary from ~/.cargo/bin)"; \
	fi
	
	@echo "=> Uninstallation finished successfully."
