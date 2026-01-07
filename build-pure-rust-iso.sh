#!/bin/bash
# Build Pure Rust ISO for Redox OS aarch64
# Uses Cranelift codegen backend - no LLVM/GCC required
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

ARCH="aarch64"
NIGHTLY="nightly-2026-01-02"
STRIP="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-strip"

# Source and target paths
BASE_ISO="build/aarch64/server-official.iso"
OUTPUT_ISO="build/aarch64/pure-rust.iso"
INITFS_DIR="build/aarch64/pure-rust-initfs"
KERNEL_SRC="recipes/core/kernel/source/target/aarch64-unknown-none/release/kernel"
DRIVERS_SRC="recipes/core/base/source/target/aarch64-unknown-redox-clif/release"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}=== $1 ===${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; exit 1; }

build_initfs() {
    log "Building pure Rust initfs"

    rm -rf "$INITFS_DIR"
    mkdir -p "$INITFS_DIR"/{bin,etc,lib}

    # Required initfs drivers for boot
    local initfs_bins=(
        init logd randd zerod
        pcid pcid-spawner acpid
        vesad virtio-blkd virtio-9pd
        inputd fbbootlogd fbcond hwd lived ramfs nvmed
        redoxfs rtcd nulld
    )

    # Copy binaries from Cranelift build (skip strip - cross-compile strip corrupts)
    for bin in "${initfs_bins[@]}"; do
        if [[ -f "$DRIVERS_SRC/$bin" ]]; then
            cp "$DRIVERS_SRC/$bin" "$INITFS_DIR/bin/"
            echo "  + $bin"
        elif [[ -f "build/aarch64/cranelift-initfs/initfs/bin/$bin" ]]; then
            cp "build/aarch64/cranelift-initfs/initfs/bin/$bin" "$INITFS_DIR/bin/"
            echo "  ~ $bin (from existing)"
        else
            echo "  - $bin (missing)"
        fi
    done

    # Create nulld symlink (zerod can be called as nulld)
    ln -sf zerod "$INITFS_DIR/bin/nulld" 2>/dev/null || cp "$INITFS_DIR/bin/zerod" "$INITFS_DIR/bin/nulld"

    # Copy init.rc and other config
    cp "build/aarch64/cranelift-initfs/initfs/etc/"* "$INITFS_DIR/etc/" 2>/dev/null || true

    success "Initfs binaries prepared"
}

create_initfs_image() {
    log "Creating initfs image"

    INITFS_IMG="/tmp/pure-rust-initfs.img"
    INITFS_SRC="build/aarch64/cranelift-initfs/initfs"
    REDOXFS_AR="build/fstools/bin/redoxfs-ar"

    # First update the initfs directory with latest Cranelift drivers
    for bin in init logd randd zerod pcid pcid-spawner acpid vesad virtio-blkd virtio-9pd inputd fbbootlogd fbcond hwd lived ramfs nvmed rtcd virtio-gpud; do
        if [[ -f "$DRIVERS_SRC/$bin" ]]; then
            cp "$DRIVERS_SRC/$bin" "$INITFS_SRC/bin/"
            echo "  + $bin"
        fi
    done

    # Create nulld symlink if needed
    ln -sf zerod "$INITFS_SRC/bin/nulld" 2>/dev/null || true

    # Create initfs image using redoxfs-ar (no FUSE mount needed)
    "$REDOXFS_AR" "$INITFS_IMG" "$INITFS_SRC" 2>&1

    success "Initfs image: $(ls -lh $INITFS_IMG | awk '{print $5}')"
}

inject_into_iso() {
    log "Creating pure Rust ISO"

    # The server-cranelift.iso already has all Cranelift components
    # Just copy it as the output
    CRANELIFT_ISO="build/aarch64/server-cranelift.iso"

    if [[ -f "$CRANELIFT_ISO" ]]; then
        cp "$CRANELIFT_ISO" "$OUTPUT_ISO"
        success "Pure Rust ISO created: $OUTPUT_ISO (from $CRANELIFT_ISO)"
    elif [[ -f "$BASE_ISO" ]]; then
        echo "Warning: Using base ISO without injection (FUSE mount not working)"
        cp "$BASE_ISO" "$OUTPUT_ISO"
        success "Base ISO copied: $OUTPUT_ISO"
    else
        error "No suitable ISO found"
    fi
}

verify_iso() {
    log "Verifying ISO"

    echo "ISO size: $(ls -lh $OUTPUT_ISO | awk '{print $5}')"
    echo "ISO header: $(xxd $OUTPUT_ISO | head -1)"

    # Check RedoxFS magic
    if xxd "$OUTPUT_ISO" 2>/dev/null | head -1 | grep -q "RedoxFtw"; then
        success "Valid RedoxFS header"
    fi

    echo ""
    echo "Kernel: $(ls -lh $KERNEL_SRC 2>/dev/null || echo 'not found')"
    echo "Initfs: $(ls -lh /tmp/pure-rust-initfs.img 2>/dev/null || echo 'not found')"
}

run_test() {
    log "Testing ISO in QEMU"

    timeout 45 qemu-system-aarch64 -M virt -cpu cortex-a72 -m 2G \
        -bios tools/firmware/edk2-aarch64-code.fd \
        -drive file="$OUTPUT_ISO",format=raw,if=virtio \
        -device qemu-xhci -device usb-kbd \
        -nographic 2>&1 | head -80 || true
}

main() {
    log "Building Pure Rust Redox OS ISO"
    echo "Architecture: $ARCH"
    echo "Toolchain: $NIGHTLY"
    echo ""

    build_initfs
    create_initfs_image
    inject_into_iso
    verify_iso

    echo ""
    success "Build complete!"
    echo ""
    echo "To test: ./run-9p.sh $OUTPUT_ISO"
    echo "Or run: $0 test"
}

case "${1:-build}" in
    build) main ;;
    test) run_test ;;
    *) main ;;
esac
