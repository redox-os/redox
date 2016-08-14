#[macro_use]
extern crate cake;

const QEMU: &'static str = "qemu-system-x86_64";

const LS_FLAGS: &'static [&'static str] = &["-a", "/"];

build! {
    // ---- COMMANDS ----
    start(harddrive) => {},
    list(kernel_list) => {},
    run(bochs) => {},
    clean() => cmd!("rm", "-rf", "build/*"),

    // ---- RECIPES ----
    bochs(harddrive) => cmd!("bochs", "-f", "bochs.x86_64"),
    qemu(harddrive) => cmd!(QEMU,
        "-serial", "mon:stdio",
        "-drive", "file=build/harddrive.bin,format=raw,index=0,media=disk"),
    libkernel() => cmd!("cargo", "rustc", "--", "-C", "lto"),
    kernel(libkernel) => cmd!("ld",
        "-m", "elf_x86_64",
        "--gc-sections",
        "-z", "max-page-size=0x1000",
        "-T bootloader/x86/kernel.ld",
        "-o", "build/kernel.in", "build/libkernel.a"),
    kernel_list(kernel) => cmd!("objdump",
        "-C", "-M", "intel",
        "-D", "build/kernel.bin",
        ">", "build/kernel.list"),
    harddrive(kernel) => cmd!("nasm",
        "-f", "bin",
        "-o", "build/harddrive.bin",
        "-D", "ARCH_x86_64", "-ibootloader/x86/", "-ibuild/",
        "bootloader/x86/harddrive.asm"),
}
