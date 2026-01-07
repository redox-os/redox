#!/bin/bash
# ============================================================================
# Redox OS - Cranelift Build System (Pure Rust Toolchain)
# ============================================================================
#
# Default architecture: aarch64 (ARM64)
# x86-specific code is marked with _x86 suffix
#
# This script builds Redox OS using a pure Rust toolchain:
#   - Cranelift codegen backend (instead of LLVM)
#   - Rust libm (instead of openlibm C library)
#   - rust-lld linker (instead of GNU ld/gcc)
#   - LLVM tools from Rust (ar, strip, objcopy)
#
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# ============================================================================
# Configuration - aarch64 is the default!
# ============================================================================

# Architecture: aarch64 (default), x86_64 (legacy, use ARCH_x86=1)
if [ -n "$ARCH_x86" ] || [ "$ARCH" = "x86_64" ]; then
    ARCH="x86_64"
    TARGET_KERNEL="x86_64-unknown-none"
    TARGET_USER="x86_64-unknown-redox"
    # x86_64 NOTE: Cranelift has better x86_64 support but aarch64 is our goal
else
    ARCH="${ARCH:-aarch64}"
    TARGET_KERNEL="aarch64-unknown-none"
    TARGET_USER="aarch64-unknown-redox"
    # aarch64 NOTE: max-atomic-width=64 (Cranelift limitation for 128-bit atomics)
fi

CONFIG="${CONFIG:-server}"
NIGHTLY="${NIGHTLY:-nightly-2026-01-02}"
CRANELIFT_LIB="${CRANELIFT_LIB:-/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib}"

# Detect host architecture for toolchain path
HOST_ARCH="$(uname -m)"
case "$HOST_ARCH" in
    arm64) HOST_ARCH="aarch64" ;;
    x86_64) HOST_ARCH="x86_64" ;;
esac
HOST_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$HOST_OS" in
    darwin) HOST_TRIPLE="${HOST_ARCH}-apple-darwin" ;;
    linux) HOST_TRIPLE="${HOST_ARCH}-unknown-linux-gnu" ;;
    *) HOST_TRIPLE="${HOST_ARCH}-unknown-${HOST_OS}" ;;
esac

TOOLCHAIN_LIB="$HOME/.rustup/toolchains/${NIGHTLY}-${HOST_TRIPLE}/lib"
RUST_SYSROOT="$HOME/.rustup/toolchains/${NIGHTLY}-${HOST_TRIPLE}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ============================================================================
# Helper Functions
# ============================================================================

log() { echo -e "${CYAN}=== $1 ===${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; exit 1; }
info() { echo -e "${BOLD}$1${NC}"; }

# ============================================================================
# Pure Rust Tool Detection
# ============================================================================

setup_rust_tools() {
    log "Setting up Pure Rust toolchain"

    # LLVM tools bundled with Rust
    LLVM_TOOLS_DIR="$RUST_SYSROOT/lib/rustlib/${HOST_TRIPLE}/bin"

    if [ -d "$LLVM_TOOLS_DIR" ]; then
        export AR="$LLVM_TOOLS_DIR/llvm-ar"
        export STRIP="$LLVM_TOOLS_DIR/llvm-strip"
        export OBJCOPY="$LLVM_TOOLS_DIR/llvm-objcopy"
        export NM="$LLVM_TOOLS_DIR/llvm-nm"
        export RANLIB="$LLVM_TOOLS_DIR/llvm-ar s"
    else
        # Fallback to system LLVM tools
        export AR="$(which llvm-ar 2>/dev/null || which ar)"
        export STRIP="$(which llvm-strip 2>/dev/null || which strip)"
        export OBJCOPY="$(which llvm-objcopy 2>/dev/null || which objcopy)"
        export NM="$(which llvm-nm 2>/dev/null || which nm)"
        export RANLIB="$(which llvm-ranlib 2>/dev/null || which ranlib)"
    fi

    # rust-lld linker (no GCC needed!)
    export RUST_LLD="$RUST_SYSROOT/lib/rustlib/${HOST_TRIPLE}/bin/rust-lld"
    if [ ! -f "$RUST_LLD" ]; then
        RUST_LLD="$(which rust-lld 2>/dev/null || which ld.lld 2>/dev/null || echo 'rust-lld')"
    fi

    # Disable C compiler for host builds where possible
    # For recipes that absolutely need CC, they can override
    export CC_FOR_BUILD="${CC_FOR_BUILD:-cc}"
    export CXX_FOR_BUILD="${CXX_FOR_BUILD:-c++}"

    # macOS compatibility: ensure GNU tools available
    if [ "$HOST_OS" = "darwin" ]; then
        export PATH="/opt/homebrew/opt/m4/bin:/opt/homebrew/opt/autoconf/bin:/opt/homebrew/opt/automake/bin:$PATH"
    fi

    info "  AR: $AR"
    info "  STRIP: $STRIP"
    info "  RUST_LLD: $RUST_LLD"
}

# ============================================================================
# Cranelift Backend Setup
# ============================================================================

setup_cranelift() {
    log "Setting up Cranelift backend"

    # Verify Cranelift library exists
    if [ ! -f "$CRANELIFT_LIB" ]; then
        error "Cranelift library not found at $CRANELIFT_LIB
Build it with:
  cd /opt/other/rustc_codegen_cranelift
  ./y.sh prepare
  ./y.sh build --sysroot clif"
    fi

    # Verify nightly toolchain
    if [ ! -d "$TOOLCHAIN_LIB" ]; then
        error "Rust toolchain not found at $TOOLCHAIN_LIB
Install with: rustup toolchain install $NIGHTLY"
    fi

    # Export Cranelift environment
    export DYLD_LIBRARY_PATH="$TOOLCHAIN_LIB:$DYLD_LIBRARY_PATH"
    export LD_LIBRARY_PATH="$TOOLCHAIN_LIB:$LD_LIBRARY_PATH"
    export RUSTFLAGS="-Zcodegen-backend=$CRANELIFT_LIB"
    export RUSTUP_TOOLCHAIN="$NIGHTLY"

    # Disable sccache - interferes with custom codegen backends
    unset RUSTC_WRAPPER
    unset CC_WRAPPER

    # Enable Cranelift in cookbook
    export COOKBOOK_CRANELIFT=1

    success "Cranelift backend ready"
}

# ============================================================================
# Target Specifications
# ============================================================================

create_target_specs() {
    log "Creating target specifications for $ARCH"

    mkdir -p tools

    if [ "$ARCH" = "aarch64" ]; then
        # aarch64 kernel target (freestanding)
        cat > tools/${TARGET_KERNEL}.json << 'EOF'
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

        # aarch64 userspace target (Redox)
        cat > tools/${TARGET_USER}-clif.json << 'EOF'
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
    else
        # x86_64 kernel target (freestanding) - legacy _x86 support
        cat > tools/${TARGET_KERNEL}.json << 'EOF'
{
    "arch": "x86_64",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "disable-redzone": true,
    "executables": true,
    "features": "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,+soft-float",
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "x86_64-unknown-none",
    "max-atomic-width": 64,
    "panic-strategy": "abort",
    "relocation-model": "static",
    "target-pointer-width": 64
}
EOF

        # x86_64 userspace target - legacy _x86 support
        cat > tools/${TARGET_USER}-clif.json << 'EOF'
{
    "arch": "x86_64",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "env": "relibc",
    "executables": true,
    "has-rpath": true,
    "linker": "rust-lld",
    "linker-flavor": "ld.lld",
    "llvm-target": "x86_64-unknown-redox",
    "max-atomic-width": 64,
    "os": "redox",
    "panic-strategy": "abort",
    "position-independent-executables": true,
    "relro-level": "full",
    "target-family": ["unix"],
    "target-pointer-width": 64
}
EOF
    fi

    success "Target specs created in tools/"
}

# ============================================================================
# Rebuild Cookbook (host tool, doesn't use Cranelift)
# ============================================================================

rebuild_cookbook() {
    log "Rebuilding cookbook"

    # Save and unset Cranelift flags (cookbook is a host tool)
    local saved_rustflags="$RUSTFLAGS"
    local saved_toolchain="$RUSTUP_TOOLCHAIN"
    unset RUSTFLAGS
    unset RUSTUP_TOOLCHAIN

    cargo build --release

    # Restore Cranelift flags
    export RUSTFLAGS="$saved_rustflags"
    export RUSTUP_TOOLCHAIN="$saved_toolchain"

    success "Cookbook rebuilt"
}

# ============================================================================
# Build Commands
# ============================================================================

build_kernel() {
    log "Building kernel for $ARCH with Cranelift"

    cd recipes/core/kernel/source

    # Copy target spec
    cp "$SCRIPT_DIR/tools/${TARGET_KERNEL}.json" .

    local kernel_rustflags="$RUSTFLAGS"

    # x86_64 needs linker script
    if [ "$ARCH" = "x86_64" ]; then
        kernel_rustflags="$kernel_rustflags -C relocation-model=static -C link-arg=-Tlinkers/x86_64.ld"
    fi

    RUSTFLAGS="$kernel_rustflags" \
    cargo build \
        --target ${TARGET_KERNEL}.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128

    local kernel_path="target/${TARGET_KERNEL}/release/kernel"
    if [ -f "$kernel_path" ]; then
        success "Kernel built: $(ls -lh $kernel_path | awk '{print $5}')"
    else
        # Try alternate name
        kernel_path="target/${TARGET_KERNEL}/release/redox_kernel"
        if [ -f "$kernel_path" ]; then
            success "Kernel built: $(ls -lh $kernel_path | awk '{print $5}')"
        else
            error "Kernel build failed"
        fi
    fi

    cd "$SCRIPT_DIR"
}

build_relibc() {
    log "Building relibc for $ARCH with Cranelift"

    cd recipes/core/relibc/source

    # Copy target spec
    cp "$SCRIPT_DIR/tools/${TARGET_USER}-clif.json" .

    # Build relibc (Rust code only - pure Rust math via libm crate)
    # NOTE: With rust-math feature, openlibm (C) is not needed!
    CARGO_TARGET_DIR="$(pwd)/target" \
    cargo build \
        --target ${TARGET_USER}-clif.json \
        --release \
        -Z build-std=core,alloc \
        -Zbuild-std-features=compiler_builtins/no-f16-f128

    local relibc_path="target/${TARGET_USER}-clif/release/librelibc.a"
    if [ -f "$relibc_path" ]; then
        success "relibc built: $(ls -lh $relibc_path | awk '{print $5}')"
    else
        error "relibc build failed"
    fi

    # Extract CRT objects
    cd "target/${TARGET_USER}-clif/release"
    for lib in crt0 crti crtn; do
        if [ -f "lib${lib}.a" ]; then
            $AR x "lib${lib}.a" 2>/dev/null || true
            # Combine objects
            local objs=$(ls ${lib}*.o 2>/dev/null | tr '\n' ' ')
            if [ -n "$objs" ]; then
                $RUST_LLD -flavor gnu -r -o "${lib}.o" $objs 2>/dev/null || true
            fi
            rm -f ${lib}-*.o 2>/dev/null
        fi
    done

    cd "$SCRIPT_DIR"
}

build_relibc_with_rust_math() {
    log "Building relibc with pure Rust math (no openlibm)"

    # First, integrate math_libm.rs if not already done
    local math_src="$SCRIPT_DIR/contrib/pure-rust/math_libm.rs"
    local relibc_math="$SCRIPT_DIR/recipes/core/relibc/source/src/math_libm.rs"

    if [ -f "$math_src" ] && [ ! -f "$relibc_math" ]; then
        info "Integrating Rust math library wrapper..."
        cp "$math_src" "$relibc_math"
    fi

    build_relibc
}

build_drivers() {
    log "Building base drivers for $ARCH with Cranelift"

    cd recipes/core/base/source

    # Copy target spec
    cp "$SCRIPT_DIR/tools/${TARGET_USER}-clif.json" .

    # Setup sysroot from relibc
    local sysroot="$SCRIPT_DIR/build/$ARCH/sysroot"
    mkdir -p "$sysroot"/{lib,include}

    local relibc_dir="$SCRIPT_DIR/recipes/core/relibc/source/target/${TARGET_USER}-clif/release"
    if [ -f "$relibc_dir/librelibc.a" ]; then
        cp "$relibc_dir/librelibc.a" "$sysroot/lib/libc.a"
        for obj in crt0.o crti.o crtn.o; do
            [ -f "$relibc_dir/$obj" ] && cp "$relibc_dir/$obj" "$sysroot/lib/"
        done
        # Create stub libraries
        $AR rcs "$sysroot/lib/libpthread.a"
        $AR rcs "$sysroot/lib/libdl.a"
        $AR rcs "$sysroot/lib/librt.a"
        $AR rcs "$sysroot/lib/libm.a"
    fi

    export CARGO_TARGET_$(echo ${TARGET_USER}-clif | tr '[:lower:]-' '[:upper:]_')_LINKER=rust-lld

    # Core drivers to build
    local drivers=(
        init audiod ipcd ptyd logd randd zerod
        pcid pcid-spawner acpid
        vesad virtio-blkd virtio-9pd
    )

    RUSTFLAGS="$RUSTFLAGS -L $sysroot/lib -Cpanic=abort" \
    cargo build \
        --target ${TARGET_USER}-clif.json \
        --release \
        -Z build-std=std,core,alloc,panic_abort \
        $(printf -- '-p %s ' "${drivers[@]}") 2>&1 || warn "Some drivers failed"

    local built=$(ls target/${TARGET_USER}-clif/release/{init,pcid,vesad} 2>/dev/null | wc -l)
    success "Built $built core drivers"

    cd "$SCRIPT_DIR"
}

build_all() {
    log "Full Cranelift build for $ARCH"

    rebuild_cookbook
    build_relibc
    build_kernel
    build_drivers

    success "Full build complete for $ARCH"
}

# ============================================================================
# Make Integration
# ============================================================================

make_target() {
    local target="$1"
    shift

    rebuild_cookbook

    export ARCH="$ARCH"
    export CONFIG_NAME="$CONFIG"
    export PODMAN_BUILD=0
    export PREFIX_BINARY=1

    make "$target" "$@"
}

# ============================================================================
# Environment Info
# ============================================================================

show_env() {
    echo ""
    info "Cranelift Build Environment"
    echo "────────────────────────────────────────────"
    echo "Architecture:     $ARCH (default: aarch64)"
    echo "Kernel target:    $TARGET_KERNEL"
    echo "Userspace target: $TARGET_USER"
    echo "Config:           $CONFIG"
    echo "Toolchain:        $NIGHTLY"
    echo ""
    echo "Cranelift:        $CRANELIFT_LIB"
    echo "RUSTFLAGS:        $RUSTFLAGS"
    echo ""
    echo "Pure Rust Tools:"
    echo "  AR:             $AR"
    echo "  STRIP:          $STRIP"
    echo "  RUST_LLD:       $RUST_LLD"
    echo "────────────────────────────────────────────"
    echo ""

    if [ "$ARCH" = "x86_64" ]; then
        warn "Using legacy x86_64 architecture"
        warn "Note: aarch64 is the default target. Use without ARCH_x86=1 for aarch64."
    else
        success "Using target architecture: aarch64"
    fi
}

# ============================================================================
# Usage
# ============================================================================

usage() {
    echo "Redox OS Cranelift Build System (Pure Rust Toolchain)"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  kernel      Build kernel with Cranelift"
    echo "  relibc      Build relibc with Cranelift"
    echo "  drivers     Build base drivers with Cranelift"
    echo "  all         Full build (kernel + relibc + drivers)"
    echo "  shell       Start shell with Cranelift environment"
    echo "  env         Show environment configuration"
    echo "  clean       Clean build artifacts"
    echo ""
    echo "Make targets (passed to make with Cranelift env):"
    echo "  r.kernel    Build kernel via cookbook"
    echo "  cr.relibc   Build relibc via cookbook"
    echo "  r.base      Build base via cookbook"
    echo "  live        Build live ISO"
    echo "  qemu        Run in QEMU"
    echo ""
    echo "Environment:"
    echo "  ARCH=aarch64    Target architecture (default: aarch64)"
    echo "  ARCH_x86=1      Use x86_64 instead of aarch64"
    echo "  CONFIG=server   Build configuration (default: server)"
    echo "  NIGHTLY=...     Rust nightly version"
    echo ""
    echo "Examples:"
    echo "  $0 kernel              # Build aarch64 kernel"
    echo "  $0 all                 # Full aarch64 build"
    echo "  ARCH_x86=1 $0 kernel   # Build x86_64 kernel (legacy)"
    echo "  $0 shell               # Enter Cranelift build shell"
    echo "  $0 live                # Build live ISO"
    echo ""
    echo "Note: aarch64 is the default architecture."
    echo "      x86_64 builds are marked as _x86 (legacy support)."
}

# ============================================================================
# Main
# ============================================================================

main() {
    setup_rust_tools
    setup_cranelift
    create_target_specs

    local cmd="${1:-help}"
    shift 2>/dev/null || true

    case "$cmd" in
        kernel)
            build_kernel
            ;;
        relibc)
            build_relibc
            ;;
        relibc-rust-math)
            build_relibc_with_rust_math
            ;;
        drivers)
            build_drivers
            ;;
        all)
            build_all
            ;;
        shell)
            show_env
            log "Starting Cranelift build shell"
            exec bash
            ;;
        env|info)
            show_env
            ;;
        clean)
            log "Cleaning build artifacts"
            rm -rf build/$ARCH/sysroot
            rm -rf recipes/core/kernel/source/target/${TARGET_KERNEL}
            rm -rf recipes/core/relibc/source/target/${TARGET_USER}-clif
            rm -rf recipes/core/base/source/target/${TARGET_USER}-clif
            success "Cleaned"
            ;;
        cookbook)
            rebuild_cookbook
            ;;
        # Make targets - pass through to make
        r.kernel|cr.relibc|r.base|r.drivers-initfs|live|qemu|repo)
            make_target "$cmd" "$@"
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            # Assume it's a make target
            if [ -n "$cmd" ]; then
                make_target "$cmd" "$@"
            else
                usage
                exit 1
            fi
            ;;
    esac
}

main "$@"
