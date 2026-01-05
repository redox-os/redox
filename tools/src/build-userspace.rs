//! Redox OS Userspace Build Tool
//! Builds the complete userspace with Rust/Cranelift, no Makefile required.

use clap::{Parser, ValueEnum};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const NIGHTLY: &str = "nightly-2026-01-02";
const CRANELIFT_LIB: &str = "/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib";

#[derive(Parser)]
#[command(name = "build-userspace")]
#[command(about = "Build Redox userspace with Rust/Cranelift")]
struct Cli {
    #[arg(long, default_value = "x86_64")]
    arch: Arch,

    #[arg(long, default_value = "cranelift")]
    backend: Backend,

    #[arg(long)]
    component: Option<Component>,

    #[arg(long)]
    verbose: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Arch {
    X86_64,
    Aarch64,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Backend {
    Cranelift,
    Llvm,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Component {
    Relibc,
    Drivers,
    Coreutils,
    All,
}

struct BuildContext {
    arch: Arch,
    backend: Backend,
    verbose: bool,
    redox_root: PathBuf,
    sysroot: PathBuf,
    target_json: PathBuf,
}

impl BuildContext {
    fn new(arch: Arch, backend: Backend, verbose: bool) -> Self {
        let redox_root = PathBuf::from("/opt/other/redox");
        let arch_str = match arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        };
        let sysroot = redox_root.join(format!("build/{}/sysroot", arch_str));
        let target_json = redox_root.join(format!("tools/{}-unknown-redox-clif.json", arch_str));

        Self { arch, backend, verbose, redox_root, sysroot, target_json }
    }

    fn target_triple(&self) -> &str {
        match self.arch {
            Arch::X86_64 => "x86_64-unknown-redox",
            Arch::Aarch64 => "aarch64-unknown-redox-clif",
        }
    }

    fn rustflags(&self) -> String {
        let mut flags = Vec::new();

        if matches!(self.backend, Backend::Cranelift) {
            flags.push(format!("-Zcodegen-backend={}", CRANELIFT_LIB));
        }

        flags.push(format!("-L{}/lib", self.sysroot.display()));
        flags.push("-Cpanic=abort".to_string());
        // Allow multiple definitions to resolve conflicts between std and relibc allocators
        flags.push("-Clink-arg=-z".to_string());
        flags.push("-Clink-arg=muldefs".to_string());

        flags.join(" ")
    }

    fn cargo_cmd(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.arg(format!("+{}", NIGHTLY));

        if matches!(self.backend, Backend::Cranelift) {
            let toolchain_lib = format!(
                "{}/.rustup/toolchains/{}-aarch64-apple-darwin/lib",
                env::var("HOME").unwrap_or_default(),
                NIGHTLY
            );
            cmd.env("DYLD_LIBRARY_PATH", &toolchain_lib);
        }

        cmd.env("RUSTFLAGS", self.rustflags());

        if self.verbose {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        }

        cmd
    }
}

fn main() {
    let cli = Cli::parse();
    let ctx = BuildContext::new(cli.arch, cli.backend, cli.verbose);

    println!("=== Redox Userspace Build ===");
    println!("Architecture: {:?}", cli.arch);
    println!("Backend: {:?}", cli.backend);
    println!("Sysroot: {}", ctx.sysroot.display());
    println!();

    if !Path::new(CRANELIFT_LIB).exists() && matches!(cli.backend, Backend::Cranelift) {
        eprintln!("Error: Cranelift backend not found at {}", CRANELIFT_LIB);
        eprintln!("Build it: cd /opt/other/rustc_codegen_cranelift && ./y.sh build");
        std::process::exit(1);
    }

    ensure_target_json(&ctx);

    match cli.component {
        Some(Component::Relibc) => build_relibc(&ctx),
        Some(Component::Drivers) => build_drivers(&ctx),
        Some(Component::Coreutils) => build_coreutils(&ctx),
        Some(Component::All) | None => {
            build_relibc(&ctx);
            build_drivers(&ctx);
            build_coreutils(&ctx);
        }
    }

    println!("\n=== Build Complete ===");
}

fn ensure_target_json(ctx: &BuildContext) {
    if matches!(ctx.arch, Arch::Aarch64) && !ctx.target_json.exists() {
        println!("Creating custom target: {}", ctx.target_json.display());
        // Use the correct data-layout that matches LLVM's aarch64-unknown-redox
        let json = r#"{
    "arch": "aarch64",
    "crt-objects-fallback": "false",
    "crt-static-default": true,
    "crt-static-respected": true,
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32",
    "dynamic-linking": true,
    "env": "relibc",
    "features": "+v8a",
    "has-rpath": true,
    "has-thread-local": true,
    "linker": "rust-lld",
    "linker-flavor": "gnu-lld",
    "llvm-target": "aarch64-unknown-redox",
    "max-atomic-width": 64,
    "os": "redox",
    "panic-strategy": "abort",
    "position-independent-executables": true,
    "relro-level": "full",
    "stack-probes": { "kind": "inline" },
    "target-family": ["unix"],
    "target-pointer-width": 64
}"#;
        fs::write(&ctx.target_json, json).expect("Failed to write target JSON");
    }
}

fn build_relibc(ctx: &BuildContext) {
    println!("=== Building relibc ===");

    let relibc_dir = ctx.redox_root.join("recipes/core/relibc/source");

    // Step 1: Generate headers
    generate_headers(ctx, &relibc_dir);

    // Step 2: Build librelibc.a
    build_librelibc(ctx, &relibc_dir);

    // Step 3: Build CRT objects
    build_crt(ctx, &relibc_dir);

    // Step 4: Build dynamic linker
    build_ld_so(ctx, &relibc_dir);

    // Step 5: Assemble sysroot
    assemble_sysroot(ctx, &relibc_dir);

    println!("relibc build complete");
}

fn generate_headers(ctx: &BuildContext, relibc_dir: &Path) {
    println!("  Generating headers...");

    let arch_str = match ctx.arch {
        Arch::X86_64 => "x86_64",
        Arch::Aarch64 => "aarch64",
    };

    let target_headers = relibc_dir.join(format!("target/{}-unknown-redox/include", arch_str));
    fs::create_dir_all(&target_headers).ok();

    // Copy static headers
    let include_src = relibc_dir.join("include");
    if include_src.exists() {
        copy_dir_recursive(&include_src, &target_headers);
    }

    // Copy openlibm headers (if we need them for compatibility)
    let openlibm_include = relibc_dir.join("openlibm/include");
    if openlibm_include.exists() {
        copy_dir_recursive(&openlibm_include, &target_headers);
    }
    let openlibm_src = relibc_dir.join("openlibm/src");
    if openlibm_src.exists() {
        for entry in fs::read_dir(&openlibm_src).unwrap().flatten() {
            if entry.path().extension().map_or(false, |e| e == "h") {
                let dest = target_headers.join(entry.file_name());
                fs::copy(entry.path(), dest).ok();
            }
        }
    }

    // Run cbindgen for each header module
    let header_dir = relibc_dir.join("src/header");
    if header_dir.exists() {
        for entry in fs::read_dir(&header_dir).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('_') { continue; }

            let cbindgen_toml = entry.path().join("cbindgen.toml");
            if cbindgen_toml.exists() {
                let out_name = name.replace('_', "/") + ".h";
                let out_path = target_headers.join(&out_name);

                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).ok();
                }

                // Combine cbindgen configs
                let globdefs = relibc_dir.join("cbindgen.globdefs.toml");
                let combined_config = format!(
                    "{}\n{}",
                    fs::read_to_string(&cbindgen_toml).unwrap_or_default(),
                    fs::read_to_string(&globdefs).unwrap_or_default()
                );

                let temp_config = relibc_dir.join(".cbindgen-combined.toml");
                fs::write(&temp_config, &combined_config).ok();

                let mod_rs = entry.path().join("mod.rs");
                let status = Command::new("cbindgen")
                    .arg(&mod_rs)
                    .arg("--config")
                    .arg(&temp_config)
                    .arg("--output")
                    .arg(&out_path)
                    .current_dir(relibc_dir)
                    .status();

                fs::remove_file(&temp_config).ok();

                if status.map_or(false, |s| s.success()) {
                    if ctx.verbose { println!("    Generated {}", out_name); }
                }
            }
        }
    }

    println!("  Headers generated: {}", target_headers.display());
}

fn build_librelibc(ctx: &BuildContext, relibc_dir: &Path) {
    println!("  Building librelibc.a...");

    let target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => ctx.target_json.to_str().unwrap(),
    };

    let mut cmd = ctx.cargo_cmd();
    cmd.arg("build")
        .arg("--release")
        .arg("--target").arg(target)
        .arg("-Zbuild-std=core,alloc")
        .arg("-Zbuild-std-features=compiler_builtins/no-f16-f128")
        .current_dir(relibc_dir);

    let status = cmd.status().expect("Failed to run cargo");
    if !status.success() {
        eprintln!("Failed to build librelibc");
        std::process::exit(1);
    }
}

fn build_crt(ctx: &BuildContext, relibc_dir: &Path) {
    println!("  Building CRT objects...");

    let target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => ctx.target_json.to_str().unwrap(),
    };

    for crt in ["crt0", "crti", "crtn"] {
        let crt_dir = relibc_dir.join(format!("src/{}", crt));
        if !crt_dir.exists() { continue; }

        let mut cmd = ctx.cargo_cmd();
        cmd.arg("build")
            .arg("--release")
            .arg("--target").arg(target)
            .arg("-Zbuild-std=core,alloc")
            .arg("-Zbuild-std-features=compiler_builtins/no-f16-f128")
            .current_dir(&crt_dir);

        cmd.status().ok();
    }
}

fn build_ld_so(ctx: &BuildContext, relibc_dir: &Path) {
    println!("  Building dynamic linker (ld_so)...");

    let target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => ctx.target_json.to_str().unwrap(),
    };

    let ld_so_dir = relibc_dir.join("ld_so");
    if !ld_so_dir.exists() { return; }

    let mut cmd = ctx.cargo_cmd();
    cmd.arg("build")
        .arg("--release")
        .arg("--target").arg(target)
        .arg("-Zbuild-std=core,alloc")
        .arg("-Zbuild-std-features=compiler_builtins/no-f16-f128")
        .current_dir(&ld_so_dir);

    cmd.status().ok();
}

fn assemble_sysroot(ctx: &BuildContext, relibc_dir: &Path) {
    println!("  Assembling sysroot...");

    let arch_str = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => "aarch64-unknown-redox-clif",
    };

    let target_dir = relibc_dir.join(format!("target/{}/release", arch_str));

    // Create sysroot directories
    let sysroot_include = ctx.sysroot.join("include");
    let sysroot_lib = ctx.sysroot.join("lib");
    fs::create_dir_all(&sysroot_include).ok();
    fs::create_dir_all(&sysroot_lib).ok();

    // Copy headers
    let headers_src = relibc_dir.join(format!("target/{}/include",
        match ctx.arch { Arch::X86_64 => "x86_64-unknown-redox", Arch::Aarch64 => "aarch64-unknown-redox" }));
    if headers_src.exists() {
        copy_dir_recursive(&headers_src, &sysroot_include);
    }

    // Copy librelibc.a as libc.a
    let librelibc = target_dir.join("librelibc.a");
    if librelibc.exists() {
        fs::copy(&librelibc, sysroot_lib.join("libc.a")).ok();
        println!("    Installed libc.a");
    }

    // Copy CRT objects
    for crt in ["crt0.o", "crti.o", "crtn.o"] {
        let src = target_dir.join(crt);
        if src.exists() {
            fs::copy(&src, sysroot_lib.join(crt)).ok();
        }
    }
    // Create crt1.o symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink("crt0.o", sysroot_lib.join("crt1.o")).ok();

    // Copy ld_so
    let ld_so = target_dir.join("ld_so");
    if ld_so.exists() {
        let ld_path = match ctx.arch {
            Arch::X86_64 => "ld64.so.1",
            Arch::Aarch64 => "ld.so.1",
        };
        fs::copy(&ld_so, sysroot_lib.join(ld_path)).ok();
    }

    // Create empty stub libraries
    for lib in ["libdl.a", "libpthread.a", "librt.a", "libm.a"] {
        let path = sysroot_lib.join(lib);
        Command::new("ar").arg("rcs").arg(&path).status().ok();
    }

    println!("  Sysroot assembled: {}", ctx.sysroot.display());
}

fn build_drivers(ctx: &BuildContext) {
    println!("\n=== Building drivers (base workspace) ===");

    let base_dir = ctx.redox_root.join("recipes/core/base/source");

    // Ensure sysroot exists
    if !ctx.sysroot.join("lib/libc.a").exists() {
        eprintln!("Warning: sysroot not found, building relibc first");
        build_relibc(ctx);
    }

    // Create stub libraries (gcc_eh, gcc_s, unwind)
    create_stub_libraries(ctx);

    let target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => ctx.target_json.to_str().unwrap(),
    };

    // Priority drivers to build
    let drivers = [
        "init", "logd", "randd", "zerod", "ramfs",
        "pcid", "pcid-spawner",
        "vesad", "fbbootlogd", "fbcond",
        "virtio-netd", "virtio-blkd",
        "ahcid", "nvmed",
        "xhcid", "usbhubd",
        "inputd", "ps2d",
    ];

    for driver in &drivers {
        println!("  Building {}...", driver);

        let mut cmd = ctx.cargo_cmd();
        cmd.arg("build")
            .arg("-p").arg(driver)
            .arg("--release")
            .arg("--target").arg(target)
            .arg("-Zbuild-std=std,core,alloc,panic_abort")
            .current_dir(&base_dir);

        let status = cmd.status();
        match status {
            Ok(s) if s.success() => println!("    {} built", driver),
            _ => println!("    {} failed (continuing)", driver),
        }
    }
}

fn build_coreutils(ctx: &BuildContext) {
    println!("\n=== Building coreutils ===");

    let coreutils_dir = ctx.redox_root.join("recipes/core/coreutils/source");
    if !coreutils_dir.exists() {
        println!("  Coreutils source not found, skipping");
        return;
    }

    let target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-redox",
        Arch::Aarch64 => ctx.target_json.to_str().unwrap(),
    };

    let mut cmd = ctx.cargo_cmd();
    cmd.arg("build")
        .arg("--release")
        .arg("--target").arg(target)
        .arg("-Zbuild-std=std,core,alloc,panic_abort")
        .current_dir(&coreutils_dir);

    let status = cmd.status();
    if status.map_or(false, |s| s.success()) {
        println!("  Coreutils built successfully");
    } else {
        println!("  Coreutils build failed");
    }
}

fn create_stub_libraries(ctx: &BuildContext) {
    let lib_dir = ctx.sysroot.join("lib");
    fs::create_dir_all(&lib_dir).ok();

    // Create unwind stubs with actual symbols
    let stub_c = r#"
typedef void *_Unwind_Context;
void *_Unwind_GetTextRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_GetDataRelBase(_Unwind_Context *ctx) { return (void*)0; }
void *_Unwind_FindEnclosingFunction(void *pc) { return (void*)0; }
unsigned long _Unwind_GetIP(_Unwind_Context *ctx) { return 0; }
unsigned long _Unwind_GetCFA(_Unwind_Context *ctx) { return 0; }
typedef int (*_Unwind_Trace_Fn)(_Unwind_Context *, void *);
int _Unwind_Backtrace(_Unwind_Trace_Fn trace, void *arg) { return 0; }
void _Unwind_Resume(void *exception_object) { }
int _Unwind_RaiseException(void *exception_object) { return 0; }
"#;

    let stub_path = lib_dir.join("unwind_stubs.c");
    fs::write(&stub_path, stub_c).ok();

    let obj_path = lib_dir.join("unwind_stubs.o");
    let clang_target = match ctx.arch {
        Arch::X86_64 => "x86_64-unknown-linux-gnu",
        Arch::Aarch64 => "aarch64-unknown-linux-gnu",
    };

    // Use -target (single dash) for compatibility with Apple clang
    let status = Command::new("clang")
        .arg("-target").arg(clang_target)
        .arg("-c")
        .arg(&stub_path)
        .arg("-o").arg(&obj_path)
        .status();

    if status.map_or(false, |s| s.success()) {
        // Create libgcc_eh.a with unwind stubs
        let lib_path = lib_dir.join("libgcc_eh.a");
        Command::new("ar").args(["rcs"]).arg(&lib_path).arg(&obj_path).status().ok();

        // Also create libgcc_s.a and libgcc.a with the same stubs
        let lib_path = lib_dir.join("libgcc_s.a");
        Command::new("ar").args(["rcs"]).arg(&lib_path).arg(&obj_path).status().ok();

        let lib_path = lib_dir.join("libgcc.a");
        Command::new("ar").args(["rcs"]).arg(&lib_path).arg(&obj_path).status().ok();

        println!("  Created unwind stub libraries");
    } else {
        // Fallback: create empty stub libraries
        for lib in ["libgcc_eh.a", "libgcc_s.a", "libgcc.a"] {
            let path = lib_dir.join(lib);
            Command::new("ar").arg("rcs").arg(&path).status().ok();
        }
        eprintln!("  Warning: Could not compile unwind stubs, using empty libs");
    }

    fs::remove_file(&stub_path).ok();
    fs::remove_file(&obj_path).ok();
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    if !src.is_dir() { return; }
    fs::create_dir_all(dst).ok();

    for entry in fs::read_dir(src).unwrap().flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).ok();
        }
    }
}
