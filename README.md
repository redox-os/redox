<p align="center">
<img alt="Redox" width="346" src="https://github.com/redox-os/assets/raw/master/logo.png">
</p>

**Redox** is an operating system written in Rust, a language with focus on safety and high performance. Redox, following the microkernel design, aims to be secure, usable, and free. Redox is inspired by previous kernels and operating systems, such as SeL4, Minix, Plan 9, and BSD.

Redox _is not_ just a kernel, it's a **full-featured Operating System**, providing packages (memory allocator, file system, display manager, core utilities, etc.) that together make up a functional and convenient operating system. You can loosely think of it as the GNU or BSD ecosystem, but in a memory safe language and with modern technology. See [this list](#ecosystem) for overview of the ecosystem.

The website can be found at https://www.redox-os.org.

Please make sure you use the **latest nightly** of `rustc` before building (for more troubleshooting, see ["Help! Redox won't compile!"](#compile-help)).

[![Travis Build Status](https://travis-ci.org/redox-os/redox.svg?branch=master)](https://travis-ci.org/redox-os/redox)
[![Downloads](https://img.shields.io/github/downloads/redox-os/redox/total.svg)](https://github.com/redox-os/redox/releases)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)
![Rust Version](https://img.shields.io/badge/rust-nightly%202017--04--22-lightgrey.svg)

## Contents

* [What it looks like](#screenshots)
* [Ecosystem](#ecosystem)
* [Help! Redox won't compile](#compile-help)
* [Contributing to Redox](#contributing)
* [Cloning, Building and running](#cloning-building-running)
 * [Quick Setup](#quick-setup)
 * [Manual Setup](#manual-setup)

## <a name="screenshots"> What it looks like </a>

<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/Desktop.png">
<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/Fancy_opacity.png">
<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/IMG_1460.PNG">

<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/Sodium_v1.png">
<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/Boot.png">
<img alt="Redox" height="150" src="https://github.com/redox-os/assets/raw/master/screenshots/IMG_1459.PNG">

## <a name="ecosystem"> Ecosystem </a>

The ecosystem and software Redox OS provides is listed below.

| Name (lexicographic order)                                                  | Maintainer
|-----------------------------------------------------------------------------|---------------------------
| [acid (kernel integration tests)](https://github.com/redox-os/acid)                              | [**@jackpot51**](https://github.com/jackpot51) (co.: [**@ticki**](https://github.com/ticki), [**@nilset](https://github.com/nilset))
| [binutils](https://github.com/redox-os/binutils)                            | [**@ticki**](https://github.com/ticki)
| [bots (custom Mattermost bots)](https://github.com/redox-os/bots)              | [**@ticki**](https://github.com/ticki)
| [cookbook](https://github.com/redox-os/cookbook)                            | [**@jackpot51**](https://github.com/jackpot51)
| [coreutils](https://github.com/redox-os/coreutils)                          | [**@ticki**](https://github.com/ticki) (co.: [**@stratact**](https://github.com/stratact))
| [extrautils](https://github.com/redox-os/extrautils)                        | [**@ticki**](https://github.com/ticki)
| [games](https://github.com/redox-os/games)                                  | [**@ticki**](https://github.com/ticki)
| [Ion (shell)](https://github.com/redox-os/ion)                              | [**@skylerberg**](https://github.com/skylerberg) & [**@jackpot51**](https://github.com/jackpot51)
| [kernel](https://github.com/redox-os/kernel)                                | [**@jackpot51**](https://github.com/jackpot51)
| [libextra](https://github.com/redox-os/libextra)                            | [**@ticki**](https://github.com/ticki)
| [libpager](https://github.com/redox-os/libpager)                            | [**@ticki**](https://github.com/ticki)
| [libstd (Redox standard library)](https://github.com/redox-os/libstd)                      | [**@jackpot51**](https://github.com/jackpot51)
| [Magnet (future package manager)](https://github.com/redox-os/magnet)       | [**@ticki**](https://github.com/ticki)
| [netutils](https://github.com/redox-os/netutils)                            | [**@jackpot51**](https://github.com/jackpot51)
| [orbclient (Orbital client)](https://github.com/redox-os/orbclient)                          | [**@jackpot51**](https://github.com/jackpot51)
| [orbdata](https://github.com/redox-os/orbdata)                              | [**@jackpot51**](https://github.com/jackpot51)
| [Orbital (windowing and compositing system)](https://github.com/redox-os/orbital)                              | [**@jackpot51**](https://github.com/jackpot51)
| [orbtk (Orbital toolkit)](https://github.com/redox-os/orbtk)                                  | [**@stratact**](https://github.com/stratact)
| [orbutils (Orbital utilities)](https://github.com/redox-os/orbutils)                            | [**@jackpot51**](https://github.com/jackpot51)
| [pkgutils (current package manager)](https://github.com/redox-os/pkgutils)  | [**@jackpot51**](https://github.com/jackpot51)
| [playbot (internal REPL bot)](https://github.com/redox-os/playbot)          | [**@ticki**](https://github.com/ticki)
| [ralloc](https://github.com/redox-os/ralloc)                                | [**@ticki**](https://github.com/ticki)
| [RANSID (Rust ANSI driver)](https://github.com/redox-os/ransid)                                | [**@jackpot51**](https://github.com/jackpot51)
| [redoxfs (old filesystem)](https://github.com/redox-os/redoxfs)             | [**@jackpot51**](https://github.com/jackpot51)
| [syscall](https://github.com/redox-os/syscall)                              | [**@jackpot51**](https://github.com/jackpot51)
| [Sodium (Vim-inspired text editor)](https://github.com/redox-os/sodium)                       | [**@ticki**](https://github.com/ticki)
| [userutils](https://github.com/redox-os/userutils)                          | [**@jackpot51**](https://github.com/jackpot51)
| [TFS (ticki filesystem)](https://github.com/ticki/tfs)                            | [**@ticki**](https://github.com/ticki)
| [The Redox book](https://github.com/redox-os/book)                          | [**@ticki**](https://github.com/ticki)
| [The old kernel](https://github.com/redox-os/old)                           | **abandoned**
| [ZFS](https://github.com/redox-os/zfs)                                      | **abandoned, superseded by TFS**

## <a name="compile-help"> Help! Redox won't compile! </a>

Sometimes things go wrong when compiling. Try the following before opening an issue:

1. Run `make clean`.
2. Run `git clean -Xfd`.
3. Make sure you have **the latest version of Rust nightly!** ([rustup.rs](https://www.rustup.rs) is recommended for managing Rust versions. If you already have it, run `rustup`).
4. Update **GNU Make**, **NASM** and **QEMU/VirtualBox**.
5. Pull the upstream master branch (`git remote add upstream git@github.com:redox-os/redox.git; git pull upstream master`).
6. Update submodules (`git submodule update --recursive --init`).

and then rebuild!

## <a name="contributing"> Contributing to Redox </a>

If you're interested in this project, and you'd like to help us out, [here](CONTRIBUTING.md) is a list of ways you can do just that.

## <a name="cloning-building-running"> Cloning, Building and Running </a>

Redox is big, even compressed. Downloading the full history may take a lot of bandwidth, and can even be costly on some data plans. Clone at your own risk!

### <a name="quick-setup" /> Quick Setup </a>

```bash
$ cd path/to/your/projects/folder/

# Run bootstrap setup
$ curl -sf https://raw.githubusercontent.com/redox-os/redox/master/bootstrap.sh -o bootstrap.sh && bash -e bootstrap.sh

# Build Redox
$ make all

# Launch using QEMU
$ make qemu
# Launch using QEMU without using KVM (Kernel Virtual Machine). Try if QEMU gives an error.
$ make qemu kvm=no
```

#### QEMU with KVM

To use QEMU with KVM (kernel-based virtual Machine), which is faster than without KVM, you need a CPU with Intel® Virtualization Technology (Intel® VT) or AMD Virtualization™ (AMD-V™) support. Most systems have this disabled by default, so you may need to reboot, go into the BIOS, and enable it.

### <a name="manual-setup"> Manual Setup </a>

To manually clone, build and run Redox using a Unix-based host, run the following commands (with exceptions, be sure to read the comments):
```bash
$ cd path/to/your/projects/folder/

# HTTPS
$ git clone https://github.com/redox-os/redox.git --origin upstream --recursive
# SSH
$ git clone git@github.com:redox-os/redox.git --origin upstream --recursive

$ cd redox/

# Install/update dependencies
$ ./bootstrap.sh -d

# Install rustup.rs
$ curl https://sh.rustup.rs -sSf | sh

# Set override toolchain to nightly build
$ rustup override set nightly

# For successive builds start here. If this is your first build, just continue

# Update git submodules
$ git submodule update --recursive --init

# Build Redox
$ make all

# Launch using QEMU
$ make qemu

# Launch using QEMU without using KVM (Kernel Virtual Machine). Try if QEMU gives an error.
$ make qemu kvm=no

# Launch using QEMU without using KVM (Kernel Virtual Machine) nor Graphics
make qemu kvm=no vga=no
```

