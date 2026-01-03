# ArmyLinux - Alpine-compatible distribution with armybox
#
# This Dockerfile creates a minimal Linux distribution that is compatible
# with Alpine Linux but uses armybox instead of BusyBox.
#
# Build:
#   docker build -t armylinux .
#
# Run:
#   docker run -it armylinux

# =============================================================================
# Stage 1: Build armybox
# =============================================================================
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    make \
    perl \
    xz-dev \
    bzip2-dev \
    zlib-dev \
    lz4-dev \
    zstd-dev

# Copy armybox source
WORKDIR /build
COPY .. /build/

# Build static armybox binary
RUN cargo build --release --target x86_64-unknown-linux-musl \
    --no-default-features --features "std,full"

# Verify static linking
RUN file /build/target/x86_64-unknown-linux-musl/release/armybox && \
    ldd /build/target/x86_64-unknown-linux-musl/release/armybox 2>&1 | grep -q "statically linked"

# =============================================================================
# Stage 2: Create minimal root filesystem
# =============================================================================
FROM alpine:3.19 AS rootfs

# Install APK tools and base packages we want to keep
RUN apk add --no-cache \
    apk-tools \
    alpine-baselayout \
    alpine-keys \
    musl \
    libc-utils \
    scanelf \
    ssl_client \
    ca-certificates

# Remove BusyBox
RUN apk del busybox busybox-binsh || true

# Copy armybox
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/armybox /bin/armybox

# Create all armybox symlinks
RUN /bin/armybox --install -s /bin && \
    /bin/armybox --install -s /sbin && \
    /bin/armybox --install -s /usr/bin && \
    /bin/armybox --install -s /usr/sbin

# Ensure /bin/sh points to armybox
RUN ln -sf /bin/armybox /bin/sh

# Set up basic configuration
COPY distro/config/profile /etc/profile
COPY distro/config/inittab /etc/inittab

# =============================================================================
# Stage 3: Final minimal image
# =============================================================================
FROM scratch

# Copy entire filesystem from rootfs stage
COPY --from=rootfs / /

# Set environment
ENV PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
ENV HOME="/root"
ENV TERM="xterm-256color"

# Default shell
SHELL ["/bin/sh", "-c"]

# Default command
CMD ["/bin/sh"]
