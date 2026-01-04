# Redox OS Development Notes

## Cranelift Backend Experiment

Branch: `experiment/cranelift-backend`

### Goal
Test building Redox with Cranelift (pure Rust) instead of LLVM (C++) to evaluate a C++-free toolchain.

### Configuration Changes
```toml
# rust-toolchain.toml - add component
components = ["rust-src", "rustfmt", "clippy", "rustc-codegen-cranelift-preview"]

# .cargo/config.toml - add to x86_64-unknown-redox target
rustflags = ["-Zcodegen-backend=cranelift"]
```

### Kernel Build Results

| Component | Status |
|-----------|--------|
| `core` | ✅ Compiles |
| `alloc` | ✅ Compiles |
| `compiler_builtins` | ✅ Compiles (needs `-Zbuild-std-features=compiler_builtins/no-f16-f128`) |
| kernel Rust code | ❌ 50 `sym` operand errors |

### Blockers

1. **`sym` operands in inline asm** - Cranelift doesn't fully support referencing symbols in inline assembly
   - Used for: interrupt handlers, context switching, boot code
   - Tracked: https://github.com/rust-lang/rustc_codegen_cranelift/issues/1204

2. **Custom target specs** - Need `"cpu": "x86-64"` field to avoid `BadName("generic")` error

### Build Command (kernel)
```bash
RUSTFLAGS="-Zcodegen-backend=cranelift" cargo rustc \
  --bin kernel \
  --target x86_64-unknown-none \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
```

### relibc Build Results (branch: `experiment/cranelift-relibc`)

| Component | Status |
|-----------|--------|
| `core` | ✅ Compiles |
| `alloc` | ✅ Compiles |
| `compiler_builtins` | ✅ Compiles |
| Dependencies (chrono, goblin, etc.) | ✅ Compile |
| `redox-rt` | ❌ 3 `sym` operand errors |

### relibc Blockers

Same fundamental issue - `asmfunction!` macro in `redox-rt/src/lib.rs` uses `sym` operands:
- `fork_impl`, `child_hook` (process creation)
- `inner_c` (signal entry)
- `PROC_FD`, `PROC_CONTROL_STRUCT` (process control)

### Build Command (relibc)
```bash
cd recipes/core/relibc/source
RUSTFLAGS="-Zcodegen-backend=cranelift" cargo build \
  --target x86_64-unknown-redox \
  --release \
  -Z build-std=core,alloc \
  -Zbuild-std-features=compiler-builtins-mem,compiler_builtins/no-f16-f128
```

### Summary

The `sym` operand limitation in Cranelift is the sole blocker for both kernel and relibc.
- Kernel: 50 errors (interrupt handlers, context switch, boot)
- relibc: 3 errors (process/signal handling)

Once rustc_codegen_cranelift#1204 is resolved, significant progress should be possible.

### Next Steps
- Try building simple userspace apps (may have no inline asm)
- Monitor Cranelift `sym` operand support progress
