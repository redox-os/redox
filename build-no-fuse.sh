#!/bin/bash
# Build Redox OS minimal without FUSE support
set -e

export PATH="/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"

# Build without FUSE mount support (macOS doesn't have FUSE easily available)
make ARCH=aarch64 CONFIG_NAME=minimal PODMAN_BUILD=0 PREFIX_BINARY=0 FSTOOLS_NO_MOUNT=1 all
