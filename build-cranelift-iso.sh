#!/bin/bash
# Build complete Redox OS ISO using Cranelift backend (pure Rust toolchain)
# Architecture: aarch64
set -e

ARCH="${1:-aarch64}"
CONFIG="${2:-server}"
NIGHTLY="nightly-2026-01-02"
CRANELIFT_LIB="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib"
ROOT="/opt/other/redox"

export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB"

# Disable sccache - it interferes with custom codegen backends
unset RUSTC_WRAPPER
unset CC_WRAPPER

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}=== $1 ===${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; exit 1; }

check_prerequisites() {
    log "Checking prerequisites"

    [[ -f "$CRANELIFT_LIB" ]] || error "Cranelift backend not found at $CRANELIFT_LIB"
    rustup run $NIGHTLY rustc --version || error "Nightly $NIGHTLY not installed"

    # Ensure cookbook is built
    if [[ ! -f "$ROOT/target/release/repo" ]]; then
        log "Building cookbook"
        cd "$ROOT"
        cargo build --release
    fi

    success "Prerequisites OK"
}

create_target_specs() {
    log "Creating custom target specifications"

    # Kernel target (freestanding, no OS)
    cat > "$ROOT/tools/aarch64-redox-none.json" << 'EOF'
{
    "arch": "aarch64",
    "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
    "disable-redzone": true,
    "executables": true,
    "features": "+strict-align,+neon,+fp-armv8",
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "aarch64-unknown-none",
    "max-atomic-width": 64,
    "panic-strategy": "abort",
    "relocation-model": "static",
    "target-pointer-width": 64
}
EOF

    # Userspace target (Redox OS)
    cat > "$ROOT/tools/aarch64-unknown-redox-clif.json" << 'EOF'
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
    "position-independent-executables": true,
    "relro-level": "full",
    "target-family": ["unix"],
    "target-pointer-width": 64
}
EOF

    success "Target specs created"
}

build_relibc() {
    log "Building relibc with Cranelift"
    cd "$ROOT/recipes/core/relibc/source"

    RELIBC_OUT="target/aarch64-unknown-redox-clif/release"

    # Check if relibc is already built
    if [[ -f "$RELIBC_OUT/librelibc.a" ]]; then
        success "relibc already built: $(ls -lh $RELIBC_OUT/librelibc.a | awk '{print $5}')"

        # Just extract CRT objects if needed
        cd "$RELIBC_OUT"
        for lib in crt0 crti crtn; do
            if [[ -f "lib${lib}.a" && ! -f "${lib}.o" ]]; then
                ar x "lib${lib}.a"
                OBJS=$(ls ${lib}*.o 2>/dev/null | tr '\n' ' ')
                if [[ -n "$OBJS" ]]; then
                    ld.lld -r -o "${lib}.o" $OBJS 2>/dev/null || true
                fi
                rm -f ${lib}-*.o 2>/dev/null
            fi
        done
        return
    fi

    # Copy target spec
    cp "$ROOT/tools/aarch64-unknown-redox-clif.json" .

    # Set TARGET for relibc Makefile
    export TARGET="aarch64-unknown-redox-clif"
    export CARGO="rustup run $NIGHTLY cargo"
    export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB"
    export CARGOFLAGS="-Z build-std=core,alloc,compiler_builtins"

    # Build headers first
    make headers TARGET=$TARGET

    # Build libs
    CARGO_TARGET_DIR="$ROOT/recipes/core/relibc/source/target" \
    $CARGO build \
        --target aarch64-unknown-redox-clif.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler_builtins/no-f16-f128

    # Extract CRT objects
    cd "$RELIBC_OUT"

    # Extract and combine CRT objects
    for lib in crt0 crti crtn; do
        if [[ -f "lib${lib}.a" ]]; then
            ar x "lib${lib}.a"
            OBJS=$(ls ${lib}*.o 2>/dev/null | tr '\n' ' ')
            if [[ -n "$OBJS" ]]; then
                ld.lld -r -o "${lib}.o" $OBJS 2>/dev/null || true
            fi
            rm -f ${lib}-*.o 2>/dev/null
        fi
    done

    success "relibc built: $(ls -lh librelibc.a 2>/dev/null | awk '{print $5}')"
}

setup_sysroot() {
    log "Setting up Cranelift sysroot"

    SYSROOT="$ROOT/build/$ARCH/cranelift-sysroot"
    RELIBC_DIR="$ROOT/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release"

    rm -rf "$SYSROOT"
    mkdir -p "$SYSROOT"/{lib,include,bin}

    # Copy relibc libraries
    cp "$RELIBC_DIR/librelibc.a" "$SYSROOT/lib/libc.a"
    [[ -f "$RELIBC_DIR/libc.so" ]] && cp "$RELIBC_DIR/libc.so" "$SYSROOT/lib/"

    # Copy CRT objects
    for obj in crt0.o crti.o crtn.o crt0_rust.o; do
        [[ -f "$RELIBC_DIR/$obj" ]] && cp "$RELIBC_DIR/$obj" "$SYSROOT/lib/"
    done

    # Copy headers
    cp -r "$ROOT/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/include"/* "$SYSROOT/include/" 2>/dev/null || true

    # Create empty stub libraries
    ar rcs "$SYSROOT/lib/libgcc_eh.a"
    ar rcs "$SYSROOT/lib/libgcc_s.a"
    ar rcs "$SYSROOT/lib/libpthread.a"
    ar rcs "$SYSROOT/lib/libdl.a"
    ar rcs "$SYSROOT/lib/librt.a"
    ar rcs "$SYSROOT/lib/libm.a"

    # Create unwind stubs
    cat > /tmp/unwind_stubs.c << 'STUBEOF'
typedef void *_Unwind_Context;
void *_Unwind_GetTextRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_GetDataRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_FindEnclosingFunction(void *pc) { return (void*)0; }
unsigned long _Unwind_GetIP(_Unwind_Context *ctx) { return 0; }
unsigned long _Unwind_GetCFA(_Unwind_Context *ctx) { return 0; }
unsigned long _Unwind_GetGR(_Unwind_Context *ctx, int index) { return 0; }
typedef int (*_Unwind_Trace_Fn)(_Unwind_Context *, void *);
int _Unwind_Backtrace(_Unwind_Trace_Fn trace, void *arg) { return 0; }
STUBEOF
    clang --target=aarch64-unknown-linux-gnu -c /tmp/unwind_stubs.c -o /tmp/unwind_stubs.o
    ar rcs "$SYSROOT/lib/libunwind_stubs.a" /tmp/unwind_stubs.o

    success "Sysroot created at $SYSROOT"
}

build_kernel() {
    log "Building kernel with Cranelift"
    cd "$ROOT/recipes/core/kernel/source"

    KERNEL="target/aarch64-redox-none/release/kernel"

    # Check if kernel is already built
    if [[ -f "$KERNEL" ]]; then
        success "Kernel already built: $(ls -lh $KERNEL | awk '{print $5}')"
        return
    fi

    cp "$ROOT/tools/aarch64-redox-none.json" .

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB" \
    rustup run $NIGHTLY cargo build \
        --target aarch64-redox-none.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128

    success "Kernel built: $(ls -lh $KERNEL | awk '{print $5}')"
}

build_base_drivers() {
    log "Building base drivers with Cranelift"
    cd "$ROOT/recipes/core/base/source"

    DRIVERS_DIR="target/aarch64-unknown-redox-clif/release"

    # Check if already built
    EXISTING=$(ls "$DRIVERS_DIR"/{init,pcid,audiod,ipcd} 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$EXISTING" -ge 4 ]]; then
        TOTAL=$(ls "$DRIVERS_DIR" | grep -E '^[a-z]' | grep -v '\.' | grep -v '^build$' | grep -v '^deps$' | grep -v '^examples$' | grep -v '^incremental$' | wc -l | tr -d ' ')
        success "Drivers already built: $TOTAL binaries"
        return
    fi

    SYSROOT="$ROOT/build/$ARCH/cranelift-sysroot"
    cp "$ROOT/tools/aarch64-unknown-redox-clif.json" .

    export CARGO_TARGET_AARCH64_UNKNOWN_REDOX_CLIF_LINKER=rust-lld

    # All drivers to build
    DRIVERS=(
        # Core services
        audiod ipcd ptyd init logd randd zerod
        # Network
        smolnetd
        # PCI and system
        pcid pcid-spawner acpid hwd
        # Storage
        ahcid nvmed virtio-blkd virtio-9pd ided lived ramfs usbscsid
        # Graphics
        vesad fbcond fbbootlogd virtio-gpud ihdgd
        # USB
        xhcid usbhidd usbhubd usbctl
        # Network drivers
        e1000d rtl8139d rtl8168d virtio-netd alxd ixgbed
        # Other
        inputd redoxerd
    )

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
        -L $SYSROOT/lib \
        -Cpanic=abort \
        -Clink-arg=-z -Clink-arg=muldefs \
        -Clink-arg=-lunwind_stubs \
        -Clink-arg=$SYSROOT/lib/crt0.o \
        -Clink-arg=$SYSROOT/lib/crti.o \
        -Clink-arg=$SYSROOT/lib/crtn.o" \
    rustup run $NIGHTLY cargo build \
        --target aarch64-unknown-redox-clif.json \
        --release \
        -Z build-std=std,core,alloc,panic_abort \
        $(printf -- '-p %s ' "${DRIVERS[@]}") \
        2>&1 | tee /tmp/drivers-build.log || warn "Some drivers failed to build"

    # Count successful builds
    BUILT=$(ls "$DRIVERS_DIR"/{audiod,ipcd,ptyd,pcid,vesad} 2>/dev/null | wc -l)
    success "Built $BUILT core drivers"
}

build_bootloader() {
    log "Building bootloader with Cranelift"
    cd "$ROOT/recipes/core/bootloader/source"

    # Bootloader might need special handling - check if it exists
    if [[ -f Cargo.toml ]]; then
        RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB" \
        rustup run $NIGHTLY cargo build \
            --target aarch64-unknown-uefi \
            --release \
            -Z build-std=core,alloc \
            -Zbuild-std-features=compiler-builtins-mem 2>&1 || warn "Bootloader build needs special handling"
    fi
}

build_initfs() {
    log "Building initfs components"
    cd "$ROOT/recipes/core/base-initfs/source" 2>/dev/null || {
        warn "base-initfs source not found, skipping"
        return
    }

    SYSROOT="$ROOT/build/$ARCH/cranelift-sysroot"
    cp "$ROOT/tools/aarch64-unknown-redox-clif.json" .

    # Build initfs drivers (minimal set for boot)
    INITFS_DRIVERS=(nvmed ahcid virtio-blkd virtio-9pd xhcid lived ramfs)

    RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB \
        -L $SYSROOT/lib \
        -Cpanic=abort" \
    rustup run $NIGHTLY cargo build \
        --target aarch64-unknown-redox-clif.json \
        --release \
        -Z build-std=std,core,alloc,panic_abort \
        $(printf -- '-p %s ' "${INITFS_DRIVERS[@]}") 2>&1 || warn "Some initfs drivers failed"
}

create_stage_dirs() {
    log "Creating stage directories"

    STAGE="$ROOT/build/$ARCH/cranelift-stage"
    rm -rf "$STAGE"
    mkdir -p "$STAGE"/{boot,usr/{bin,lib,lib/drivers,include},etc/pcid.d}

    # Copy kernel
    KERNEL="$ROOT/recipes/core/kernel/source/target/aarch64-redox-none/release/kernel"
    if [[ -f "$KERNEL" ]]; then
        llvm-strip -o "$STAGE/boot/kernel" "$KERNEL" 2>/dev/null || cp "$KERNEL" "$STAGE/boot/kernel"
        success "Kernel staged"
    fi

    # Copy relibc
    RELIBC_DIR="$ROOT/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release"
    cp "$RELIBC_DIR/librelibc.a" "$STAGE/usr/lib/libc.a"
    for obj in crt0.o crti.o crtn.o; do
        [[ -f "$RELIBC_DIR/$obj" ]] && cp "$RELIBC_DIR/$obj" "$STAGE/usr/lib/"
    done

    # Copy drivers
    DRIVERS_DIR="$ROOT/recipes/core/base/source/target/aarch64-unknown-redox-clif/release"
    for bin in audiod ipcd ptyd smolnetd pcid pcid-spawner inputd redoxerd; do
        [[ -f "$DRIVERS_DIR/$bin" ]] && cp "$DRIVERS_DIR/$bin" "$STAGE/usr/bin/"
    done
    for drv in ahcid nvmed virtio-blkd vesad fbcond xhcid usbhidd virtio-netd e1000d; do
        [[ -f "$DRIVERS_DIR/$drv" ]] && cp "$DRIVERS_DIR/$drv" "$STAGE/usr/lib/drivers/"
    done

    # Copy driver configs
    cp "$ROOT/recipes/core/base/source/drivers"/*.toml "$STAGE/etc/pcid.d/" 2>/dev/null || true

    success "Stage directory created"
}

create_iso() {
    log "Creating ISO image"

    ISO="$ROOT/build/$ARCH/cranelift-redox.iso"
    STAGE="$ROOT/build/$ARCH/cranelift-stage"

    # Check if redox_installer is available
    INSTALLER="$ROOT/build/fstools/bin/redox_installer"
    if [[ ! -f "$INSTALLER" ]]; then
        warn "redox_installer not found, creating raw image instead"

        # Create a simple raw filesystem image
        REDOXFS_MKFS="$ROOT/build/fstools/bin/redoxfs-mkfs"
        if [[ -f "$REDOXFS_MKFS" ]]; then
            rm -f "$ISO"
            truncate -s 512M "$ISO"
            "$REDOXFS_MKFS" "$ISO"
            success "Created raw image at $ISO"
        else
            warn "redoxfs-mkfs not found"
        fi
        return
    fi

    # Use installer to create proper ISO
    rm -f "$ISO"
    truncate -s 512M "$ISO"
    "$INSTALLER" -c "$ROOT/config/$ARCH/$CONFIG.toml" --live "$ISO"

    success "ISO created at $ISO"
}

inject_into_existing_iso() {
    log "Injecting Cranelift binaries into existing ISO"

    EXISTING_ISO="$ROOT/build/$ARCH/desktop/redox-live.iso"
    if [[ ! -f "$EXISTING_ISO" ]]; then
        warn "No existing ISO found at $EXISTING_ISO"
        return
    fi

    MOUNT_DIR="/tmp/redoxfs-cranelift-mount"
    REDOXFS="$ROOT/build/fstools/bin/redoxfs"

    mkdir -p "$MOUNT_DIR"

    # Mount existing ISO
    "$REDOXFS" "$EXISTING_ISO" "$MOUNT_DIR" || {
        warn "Cannot mount ISO (needs macFUSE)"
        return
    }

    sleep 2

    # Replace kernel
    KERNEL="$ROOT/recipes/core/kernel/source/target/aarch64-redox-none/release/kernel"
    if [[ -f "$KERNEL" ]]; then
        llvm-strip -o "$MOUNT_DIR/boot/kernel" "$KERNEL" 2>/dev/null || cp "$KERNEL" "$MOUNT_DIR/boot/kernel"
        success "Kernel replaced"
    fi

    # Replace userspace binaries
    DRIVERS_DIR="$ROOT/recipes/core/base/source/target/aarch64-unknown-redox-clif/release"
    for bin in audiod ipcd ptyd smolnetd pcid pcid-spawner inputd; do
        if [[ -f "$DRIVERS_DIR/$bin" ]]; then
            cp "$DRIVERS_DIR/$bin" "$MOUNT_DIR/usr/bin/" 2>/dev/null && echo "  Replaced $bin"
        fi
    done

    for drv in vesad virtio-netd virtio-blkd; do
        if [[ -f "$DRIVERS_DIR/$drv" ]]; then
            cp "$DRIVERS_DIR/$drv" "$MOUNT_DIR/usr/lib/drivers/" 2>/dev/null && echo "  Replaced $drv"
        fi
    done

    sync
    umount "$MOUNT_DIR" || fusermount -u "$MOUNT_DIR" 2>/dev/null
    rm -rf "$MOUNT_DIR"

    success "Injected Cranelift binaries into existing ISO"
}

print_summary() {
    echo ""
    log "Build Summary"
    echo ""

    echo "Kernel:"
    ls -lh "$ROOT/recipes/core/kernel/source/target/aarch64-redox-none/release/kernel" 2>/dev/null || echo "  Not built"

    echo ""
    echo "relibc:"
    ls -lh "$ROOT/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release/librelibc.a" 2>/dev/null || echo "  Not built"

    echo ""
    echo "Drivers:"
    ls "$ROOT/recipes/core/base/source/target/aarch64-unknown-redox-clif/release/"*.d 2>/dev/null | head -10 || echo "  None built"

    echo ""
    echo "ISO:"
    ls -lh "$ROOT/build/$ARCH/cranelift-redox.iso" 2>/dev/null || echo "  Not created"
}

usage() {
    echo "Usage: $0 [arch] [config] [component]"
    echo ""
    echo "  arch:      aarch64 (default)"
    echo "  config:    server, desktop (default: server)"
    echo "  component: all, relibc, kernel, drivers, iso (default: all)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build everything for aarch64 server"
    echo "  $0 aarch64 server     # Same as above"
    echo "  $0 aarch64 desktop    # Build for desktop config"
    echo "  $0 aarch64 server relibc  # Only build relibc"
    exit 1
}

main() {
    cd "$ROOT"

    COMPONENT="${3:-all}"

    check_prerequisites
    create_target_specs

    case "$COMPONENT" in
        relibc)
            build_relibc
            setup_sysroot
            ;;
        kernel)
            build_kernel
            ;;
        drivers)
            build_base_drivers
            ;;
        iso)
            create_stage_dirs
            create_iso
            ;;
        inject)
            inject_into_existing_iso
            ;;
        all)
            build_relibc
            setup_sysroot
            build_kernel
            build_base_drivers
            create_stage_dirs
            inject_into_existing_iso
            ;;
        *)
            usage
            ;;
    esac

    print_summary
    echo ""
    success "Build complete!"
}

main "$@"
