# iOS Package Makefile for armybox
# Requires: macOS, Theos, Rust with iOS targets

PACKAGE_NAME = armybox
VERSION = 0.1.0
ARCH = iphoneos-arm64

# Directories
PROJECT_DIR = $(shell cd ../.. && pwd)
BUILD_DIR = build
STAGING_DIR = $(BUILD_DIR)/staging
PACKAGES_DIR = packages

# Binary locations
ARMYBOX_BIN = $(PROJECT_DIR)/target/aarch64-apple-ios/release/armybox

.PHONY: all clean build package

all: package

# Build the Rust binary for iOS
build:
	@echo "Building armybox for iOS..."
	cd $(PROJECT_DIR) && \
	SDKROOT=$(shell xcrun --sdk iphoneos --show-sdk-path) \
	cargo build --release --target aarch64-apple-ios
	@echo "Build complete!"

# Create the .deb package
package: build
	@echo "Creating iOS package..."

	# Clean and create directories
	rm -rf $(BUILD_DIR) $(PACKAGES_DIR)
	mkdir -p $(STAGING_DIR)/usr/bin
	mkdir -p $(STAGING_DIR)/usr/local/bin
	mkdir -p $(STAGING_DIR)/DEBIAN
	mkdir -p $(PACKAGES_DIR)

	# Copy binary
	cp $(ARMYBOX_BIN) $(STAGING_DIR)/usr/bin/armybox
	chmod 755 $(STAGING_DIR)/usr/bin/armybox

	# Sign with ldid if available
	@if command -v ldid >/dev/null 2>&1; then \
		echo "Signing binary with ldid..."; \
		ldid -Sentitlements.plist $(STAGING_DIR)/usr/bin/armybox; \
	else \
		echo "Warning: ldid not found, binary will need to be signed on device"; \
	fi

	# Copy control files
	cp control $(STAGING_DIR)/DEBIAN/
	cp postinst $(STAGING_DIR)/DEBIAN/
	cp prerm $(STAGING_DIR)/DEBIAN/
	chmod 755 $(STAGING_DIR)/DEBIAN/postinst
	chmod 755 $(STAGING_DIR)/DEBIAN/prerm

	# Calculate installed size
	SIZE=$$(du -sk $(STAGING_DIR) | cut -f1); \
	echo "Installed-Size: $$SIZE" >> $(STAGING_DIR)/DEBIAN/control

	# Build the deb
	dpkg-deb -Zgzip -b $(STAGING_DIR) $(PACKAGES_DIR)/$(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb

	@echo ""
	@echo "Package created: $(PACKAGES_DIR)/$(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb"
	@ls -lh $(PACKAGES_DIR)/*.deb

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR) $(PACKAGES_DIR)
	cd $(PROJECT_DIR) && cargo clean --target aarch64-apple-ios

# Install to connected device via SSH
install: package
	@echo "Installing to device..."
	@if [ -z "$(DEVICE_IP)" ]; then \
		echo "Error: Set DEVICE_IP environment variable"; \
		echo "Usage: make install DEVICE_IP=192.168.1.x"; \
		exit 1; \
	fi
	scp $(PACKAGES_DIR)/*.deb root@$(DEVICE_IP):/var/mobile/
	ssh root@$(DEVICE_IP) "dpkg -i /var/mobile/$(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb"
	@echo "Installed!"
