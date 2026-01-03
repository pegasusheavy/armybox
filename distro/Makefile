# ArmyLinux Build System
# An Alpine-compatible distribution powered by armybox

SHELL := /bin/bash
.PHONY: all clean rootfs iso docker qemu help

# Configuration
DISTRO_NAME := armylinux
VERSION := 0.1.0
ARCH := x86_64
ALPINE_VERSION := 3.19

# Directories
ROOT_DIR := $(shell pwd)
ROOTFS_DIR := $(ROOT_DIR)/rootfs
ISO_DIR := $(ROOT_DIR)/iso
BUILD_DIR := $(ROOT_DIR)/build
ARMYBOX_DIR := $(ROOT_DIR)/..

# Armybox binary
ARMYBOX_BIN := $(ARMYBOX_DIR)/target/x86_64-unknown-linux-musl/release/armybox

# Default target
all: help

help:
	@echo "ArmyLinux Build System"
	@echo ""
	@echo "Targets:"
	@echo "  armybox   - Build armybox static binary"
	@echo "  rootfs    - Build root filesystem"
	@echo "  docker    - Build Docker image"
	@echo "  iso       - Build bootable ISO (requires root)"
	@echo "  qemu      - Run in QEMU"
	@echo "  clean     - Clean build artifacts"
	@echo ""
	@echo "Configuration:"
	@echo "  ARCH=$(ARCH)"
	@echo "  ALPINE_VERSION=$(ALPINE_VERSION)"

# Build armybox static binary
armybox:
	@echo "==> Building armybox static binary..."
	cd $(ARMYBOX_DIR) && make static
	@test -f $(ARMYBOX_BIN) || (echo "ERROR: armybox binary not found at $(ARMYBOX_BIN)" && exit 1)
	@echo "==> armybox binary ready: $(ARMYBOX_BIN)"
	@ls -lh $(ARMYBOX_BIN)

# Build root filesystem
rootfs: armybox
	@echo "==> Building root filesystem..."
	./scripts/build-rootfs.sh

# Build Docker image
docker: armybox
	@echo "==> Building Docker image..."
	./scripts/build-docker.sh

# Build bootable ISO
iso: rootfs
	@echo "==> Building bootable ISO..."
	./scripts/build-iso.sh

# Run in QEMU
qemu: rootfs
	@echo "==> Starting QEMU..."
	./scripts/run-qemu.sh

# Clean build artifacts
clean:
	@echo "==> Cleaning build artifacts..."
	rm -rf $(ROOTFS_DIR)/*
	rm -rf $(ISO_DIR)/*
	rm -rf $(BUILD_DIR)
	@echo "==> Clean complete"

# Deep clean (including armybox)
distclean: clean
	cd $(ARMYBOX_DIR) && cargo clean
