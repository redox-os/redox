#!/bin/bash
# Build Redox OS minimal configuration for aarch64
set -e

# Add GNU coreutils to PATH (required on macOS)
export PATH="/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"

# Build without Podman (native build)
make ARCH=aarch64 CONFIG_NAME=minimal PODMAN_BUILD=0 PREFIX_BINARY=0 all
