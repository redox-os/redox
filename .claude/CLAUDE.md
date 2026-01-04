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

### Next Steps
- Try building relibc (may have fewer inline asm requirements)
- Try building userspace apps
- Monitor Cranelift `sym` operand support progress
