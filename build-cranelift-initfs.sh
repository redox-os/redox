#!/bin/bash
# Build initfs with Cranelift backend for aarch64
set -e

ARCH="${1:-aarch64}"
NIGHTLY="nightly-2026-01-02"
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib"
ROOT="/opt/other/redox"
TARGET="aarch64-unknown-redox-clif"
KERNEL_TARGET="aarch64-redox-none"

export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB"
unset RUSTC_WRAPPER
unset CC_WRAPPER

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}=== $1 ===${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; exit 1; }

BASE_SOURCE="$ROOT/recipes/core/base/source"
RELIBC_DIR="$ROOT/recipes/core/relibc/source/target/$TARGET/release"
REDOXFS_SOURCE="$ROOT/recipes/core/redoxfs/source"
SYSROOT="$ROOT/build/$ARCH/cranelift-sysroot"
BUILD_DIR="$ROOT/build/$ARCH/cranelift-initfs"

build_initfs_binaries() {
    log "Building initfs binaries with Cranelift"
    cd "$BASE_SOURCE"

    cp "$ROOT/tools/$TARGET.json" . 2>/dev/null || true

    INITFS_BINS=(
        init logd ramfs randd zerod
        acpid fbbootlogd fbcond hwd inputd lived nvmed
        pcid pcid-spawner rtcd vesad
        virtio-blkd virtio-gpud
    )

    BUILT=0
    for bin in "${INITFS_BINS[@]}"; do
        if [[ -f "target/$TARGET/release/$bin" ]]; then
            ((BUILT++))
        fi
    done

    if [[ $BUILT -ge ${#INITFS_BINS[@]} ]]; then
        success "All ${#INITFS_BINS[@]} initfs binaries already built"
        return
    fi

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
        -L $SYSROOT/lib \
        -Cpanic=abort \
        -Clink-arg=-z -Clink-arg=muldefs \
        -Clink-arg=-lunwind_stubs \
        -Clink-arg=$SYSROOT/lib/crt0.o \
        -Clink-arg=$SYSROOT/lib/crti.o \
        -Clink-arg=$SYSROOT/lib/crtn.o" \
    rustup run $NIGHTLY cargo build \
        --target "$TARGET.json" \
        --release \
        -Z build-std=std,core,alloc,panic_abort \
        $(printf -- '-p %s ' "${INITFS_BINS[@]}") 2>&1 || warn "Some binaries failed"

    BUILT=$(ls target/$TARGET/release/{init,logd,pcid,vesad} 2>/dev/null | wc -l)
    success "Built $BUILT initfs binaries"
}

build_redoxfs() {
    log "Building redoxfs with Cranelift"
    cd "$REDOXFS_SOURCE"

    if [[ -f "target/$TARGET/release/redoxfs" ]]; then
        success "redoxfs already built"
        return
    fi

    cp "$ROOT/tools/$TARGET.json" . 2>/dev/null || true

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
        -L $SYSROOT/lib \
        -Cpanic=abort \
        -Clink-arg=-z -Clink-arg=muldefs \
        -Clink-arg=-lunwind_stubs \
        -Clink-arg=$SYSROOT/lib/crt0.o \
        -Clink-arg=$SYSROOT/lib/crti.o \
        -Clink-arg=$SYSROOT/lib/crtn.o \
        -Ctarget-feature=+crt-static" \
    rustup run $NIGHTLY cargo build \
        --target "$TARGET.json" \
        --release \
        -Z build-std=std,core,alloc,panic_abort 2>&1 || warn "redoxfs build failed"

    if [[ -f "target/$TARGET/release/redoxfs" ]]; then
        success "redoxfs built: $(ls -lh target/$TARGET/release/redoxfs | awk '{print $5}')"
    else
        warn "redoxfs binary not found"
    fi
}

build_bootstrap() {
    log "Building bootstrap static library with Cranelift"
    cd "$BASE_SOURCE"

    # Bootstrap is no_std and uses target_os = "redox"
    # It needs redox-rt from relibc

    # Create a modified target for bootstrap (no_std, target_os = redox)
    cat > "$ROOT/tools/$TARGET-bootstrap.json" << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32",
    "env": "relibc",
    "executables": true,
    "features": "+strict-align,+neon,+fp-armv8",
    "has-rpath": true,
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "aarch64-unknown-redox",
    "max-atomic-width": 64,
    "os": "redox",
    "panic-strategy": "abort",
    "position-independent-executables": false,
    "relocation-model": "static",
    "relro-level": "off",
    "target-family": ["unix"],
    "target-pointer-width": 64
}
EOF

    cp "$ROOT/tools/$TARGET-bootstrap.json" .

    # Build bootstrap static library
    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
        -Ctarget-feature=+crt-static" \
    rustup run $NIGHTLY cargo build \
        --target "$TARGET-bootstrap.json" \
        --manifest-path bootstrap/Cargo.toml \
        --release \
        -Z build-std=core,alloc,compiler_builtins \
        -Zbuild-std-features=compiler-builtins-mem 2>&1 || {
            warn "Bootstrap build failed, checking errors..."
            return 1
        }

    BOOTSTRAP_LIB="bootstrap/target/$TARGET-bootstrap/release/libbootstrap.a"
    if [[ -f "$BOOTSTRAP_LIB" ]]; then
        success "Bootstrap library built: $(ls -lh $BOOTSTRAP_LIB | awk '{print $5}')"
    else
        warn "Bootstrap library not found"
        return 1
    fi
}

link_bootstrap() {
    log "Linking bootstrap executable"
    cd "$BASE_SOURCE"

    BOOTSTRAP_LIB="bootstrap/target/$TARGET-bootstrap/release/libbootstrap.a"
    DEPS_DIR="bootstrap/target/$TARGET-bootstrap/release/deps"
    LINKER_SCRIPT="bootstrap/src/aarch64.ld"

    [[ -f "$BOOTSTRAP_LIB" ]] || error "Bootstrap library not found"
    [[ -f "$LINKER_SCRIPT" ]] || error "Linker script not found"
    [[ -d "$DEPS_DIR" ]] || error "Dependencies directory not found"

    mkdir -p "$BUILD_DIR"

    # Collect all rlib files
    RLIBS=$(find "$DEPS_DIR" -name "*.rlib" -type f | tr '\n' ' ')

    ld.lld \
        -o "$BUILD_DIR/bootstrap" \
        --gc-sections \
        -T "$LINKER_SCRIPT" \
        -z max-page-size=4096 \
        "$BOOTSTRAP_LIB" \
        $RLIBS 2>&1 || error "Bootstrap linking failed"

    if [[ -f "$BUILD_DIR/bootstrap" ]]; then
        success "Bootstrap linked: $(ls -lh $BUILD_DIR/bootstrap | awk '{print $5}')"
        file "$BUILD_DIR/bootstrap"
    fi
}

create_initfs_directory() {
    log "Creating initfs directory structure"

    rm -rf "$BUILD_DIR/initfs"
    mkdir -p "$BUILD_DIR/initfs"/{bin,lib/drivers,etc/pcid}

    # Copy and strip binaries to reduce size
    DRIVERS_DIR="$BASE_SOURCE/target/$TARGET/release"

    # Bin directory binaries
    for bin in init logd ramfs randd zerod pcid pcid-spawner fbbootlogd fbcond inputd vesad lived acpid rtcd hwd; do
        if [[ -f "$DRIVERS_DIR/$bin" ]]; then
            llvm-strip -o "$BUILD_DIR/initfs/bin/$bin" "$DRIVERS_DIR/$bin" 2>/dev/null || \
            cp "$DRIVERS_DIR/$bin" "$BUILD_DIR/initfs/bin/"
            echo "  + bin/$bin"
        else
            warn "Missing: $bin"
        fi
    done

    # Create nulld as copy of zerod
    [[ -f "$BUILD_DIR/initfs/bin/zerod" ]] && cp "$BUILD_DIR/initfs/bin/zerod" "$BUILD_DIR/initfs/bin/nulld"

    # Driver binaries (stripped)
    for drv in ahcid ided nvmed virtio-blkd virtio-gpud; do
        if [[ -f "$DRIVERS_DIR/$drv" ]]; then
            llvm-strip -o "$BUILD_DIR/initfs/lib/drivers/$drv" "$DRIVERS_DIR/$drv" 2>/dev/null || \
            cp "$DRIVERS_DIR/$drv" "$BUILD_DIR/initfs/lib/drivers/"
            echo "  + lib/drivers/$drv"
        fi
    done

    # Copy and strip redoxfs
    if [[ -f "$REDOXFS_SOURCE/target/$TARGET/release/redoxfs" ]]; then
        llvm-strip -o "$BUILD_DIR/initfs/bin/redoxfs" "$REDOXFS_SOURCE/target/$TARGET/release/redoxfs" 2>/dev/null || \
        cp "$REDOXFS_SOURCE/target/$TARGET/release/redoxfs" "$BUILD_DIR/initfs/bin/"
        echo "  + bin/redoxfs (Cranelift)"
    elif [[ -f "$ROOT/build/fstools/bin/redoxfs" ]]; then
        # Fall back to LLVM-built redoxfs (host tool, not target)
        warn "Using host redoxfs - need to cross-compile for target"
    fi

    # Copy config files
    cp "$BASE_SOURCE/init.rc" "$BUILD_DIR/initfs/etc/"
    cp "$BASE_SOURCE/init_drivers.rc" "$BUILD_DIR/initfs/etc/"
    [[ -f "$BASE_SOURCE/aarch64-unknown-redox/init_drivers.rc" ]] && \
        cp "$BASE_SOURCE/aarch64-unknown-redox/init_drivers.rc" "$BUILD_DIR/initfs/etc/"
    cp "$BASE_SOURCE/drivers/initfs.toml" "$BUILD_DIR/initfs/etc/pcid/"

    success "initfs directory created"
    ls -la "$BUILD_DIR/initfs/bin/" | head -10
}

create_initfs_image() {
    log "Creating initfs.img"

    ARCHIVER="$BUILD_DIR/initfs-tools-target/release/redox-initfs-ar"
    [[ -f "$ARCHIVER" ]] || error "Archiver not built - run build_initfs_archiver first"

    # Create the initfs image (128 MiB max to accommodate larger Cranelift binaries)
    "$ARCHIVER" "$BUILD_DIR/initfs" "$BUILD_DIR/bootstrap" -o "$BUILD_DIR/initfs.img" --max-size 134217728 2>&1 || error "Failed to create initfs.img"

    if [[ -f "$BUILD_DIR/initfs.img" ]]; then
        success "initfs.img created: $(ls -lh $BUILD_DIR/initfs.img | awk '{print $5}')"
    fi
}

inject_initfs() {
    log "Injecting Cranelift initfs into ISO"

    EXISTING_ISO="$ROOT/build/$ARCH/desktop/redox-live.iso"
    [[ -f "$EXISTING_ISO" ]] || {
        warn "No existing ISO at $EXISTING_ISO"
        return
    }

    MOUNT_DIR="/tmp/redoxfs-initfs-mount"
    REDOXFS="$ROOT/build/fstools/bin/redoxfs"

    mkdir -p "$MOUNT_DIR"

    "$REDOXFS" "$EXISTING_ISO" "$MOUNT_DIR" || {
        warn "Cannot mount ISO (needs macFUSE)"
        return
    }

    sleep 2

    # Replace initfs
    if [[ -f "$BUILD_DIR/initfs.img" ]]; then
        cp "$BUILD_DIR/initfs.img" "$MOUNT_DIR/boot/initfs"
        success "Replaced boot/initfs"
    fi

    # Also replace kernel if available
    KERNEL="$ROOT/recipes/core/kernel/source/target/$KERNEL_TARGET/release/kernel"
    if [[ -f "$KERNEL" ]]; then
        llvm-strip -o "$MOUNT_DIR/boot/kernel" "$KERNEL" 2>/dev/null || cp "$KERNEL" "$MOUNT_DIR/boot/kernel"
        success "Replaced boot/kernel"
    fi

    sync
    umount "$MOUNT_DIR" || diskutil unmount "$MOUNT_DIR" 2>/dev/null || fusermount -u "$MOUNT_DIR" 2>/dev/null
    rm -rf "$MOUNT_DIR"

    success "ISO updated with Cranelift initfs"
}

build_initfs_archiver() {
    log "Building initfs archiver (host tool)"

    INITFS_AR="$BASE_SOURCE/initfs/tools"

    # Build BEFORE any changes to workspace Cargo.lock
    cd "$INITFS_AR"
    CARGO_TARGET_DIR="$BUILD_DIR/initfs-tools-target" \
    cargo +stable build --release --bin redox-initfs-ar 2>&1 || {
        CARGO_TARGET_DIR="$BUILD_DIR/initfs-tools-target" \
        cargo build --release --bin redox-initfs-ar 2>&1 || error "Failed to build initfs archiver"
    }

    ARCHIVER="$BUILD_DIR/initfs-tools-target/release/redox-initfs-ar"
    [[ -f "$ARCHIVER" ]] || error "Archiver not found at $ARCHIVER"
    success "Archiver built: $(ls -lh $ARCHIVER | awk '{print $5}')"
}

main() {
    cd "$ROOT"

    [[ -f "$CRANELIFT_LIB" ]] || error "Cranelift backend not found"
    [[ -d "$SYSROOT" ]] || error "Sysroot not found at $SYSROOT - run build-cranelift-iso.sh first"

    # Build archiver first before modifying workspace
    build_initfs_archiver
    build_initfs_binaries
    build_redoxfs
    build_bootstrap
    link_bootstrap
    create_initfs_directory
    create_initfs_image
    inject_initfs

    echo ""
    log "Build Summary"
    echo "initfs.img: $(ls -lh $BUILD_DIR/initfs.img 2>/dev/null | awk '{print $5}' || echo 'not created')"
    echo ""
    success "Done!"
}

main "$@"
