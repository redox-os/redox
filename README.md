<p align="center">
<img alt="Redox" width="346" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/logos/redox/logo.png">
</p>

**Redox** is an operating system written in Rust, a language with focus on safety and high performance. Redox, following the microkernel design, aims to be secure, usable, and free. Redox is inspired by previous kernels and operating systems, such as SeL4, MINIX, Plan 9, and BSD.

Redox _is not_ just a kernel, it's a **full-featured Operating System**, providing packages (memory allocator, file system, display manager, core utilities, etc.) that together make up a functional and convenient operating system. You can loosely think of it as the GNU or BSD ecosystem, but in a memory safe language and with modern technology. See [this list](#ecosystem) for overview of the ecosystem.

The website can be found at https://www.redox-os.org.

Please make sure you use the **latest nightly** of `rustc` before building (for more troubleshooting, see ["Help! Redox won't compile!"](#compile-help)).

[![Travis Build Status](https://travis-ci.org/redox-os/redox.svg?branch=master)](https://travis-ci.org/redox-os/redox)
[![Downloads](https://img.shields.io/github/downloads/redox-os/redox/total.svg)](https://gitlab.redox-os.org/redox-os/redox/tags)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
![Rust Version](https://img.shields.io/badge/rust-nightly%202017--10--03-lightgrey.svg)

## Contents

* [What it looks like](#screenshots)
* [Ecosystem](#ecosystem)
* [Help! Redox won't compile](#compile-help)
* [Contributing to Redox](#contributing)
* [Cloning, Building and running](#cloning-building-running)
 * [Quick Setup](#quick-setup)
 * [Manual Setup](#manual-setup)
 * [Setup Using Docker](#setup-using-docker)

## <a name="screenshots"> What it looks like </a>

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Senza%20titolo.jpeg">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/redox running.jpeg">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/IMG_1460.PNG">

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Sodium_v2.PNG">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Boot.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/IMG_1459.PNG">

## <a name="ecosystem"> Ecosystem </a>

The ecosystem and software Redox OS provides is listed below.

| Name (lexicographic order)                                                           | Maintainer
|--------------------------------------------------------------------------------------|---------------------------
| [acid (kernel integration tests)](https://gitlab.redox-os.org/redox-os/acid)         | **@jackpot51** **@NilSet**
| [binutils](https://gitlab.redox-os.org/redox-os/binutils)                            | **vacant**
| [cookbook](https://gitlab.redox-os.org/redox-os/cookbook)                            | **@jackpot51** **@ids1024** **@sajattack**
| [coreutils](https://gitlab.redox-os.org/redox-os/coreutils)                          | **vacant**
| [extrautils](https://gitlab.redox-os.org/redox-os/extrautils)                        | **vacant**
| [games](https://gitlab.redox-os.org/redox-os/games)                                  | **@enrico** (AKA **@HenryTheCat**) **@fabiao**
| [Ion (shell)](https://gitlab.redox-os.org/redox-os/ion)                              | **@mmstick** **@stratact**
| [ipcd](https://gitlab.redox-os.org/redox-os/ipcd)                                    | **@jD91mZM2**
| [kernel](https://gitlab.redox-os.org/redox-os/kernel)                                | **@jackpot51**
| [libextra](https://gitlab.redox-os.org/redox-os/libextra)                            | **vacant**
| [libpager](https://gitlab.redox-os.org/redox-os/libpager)                            | **vacant**
| [netstack](https://gitlab.redox-os.org/redox-os/netstack)                            | **@batonius** **@dlrobertson**
| [netutils](https://gitlab.redox-os.org/redox-os/netutils)                            | **@jackpot51**
| [orbclient (Orbital client)](https://gitlab.redox-os.org/redox-os/orbclient)         | **@jackpot51** **@FloVanGH**
| [orbdata](https://gitlab.redox-os.org/redox-os/orbdata)                              | **@jackpot51**
| [orbgame (Orbital 2D game engine)](https://gitlab.redox-os.org/redox-os/orbgame)     | **@FloVanGH**
| [Orbital (windowing and compositing system)](https://gitlab.redox-os.org/redox-os/orbital) | **@jackpot51**
| [orbtk (Orbital toolkit)](https://gitlab.redox-os.org/redox-os/orbtk)                | **@FloVanGH**
| [orbutils (Orbital utilities)](https://gitlab.redox-os.org/redox-os/orbutils)        | **@jackpot51**
| [pkgutils (current package manager)](https://gitlab.redox-os.org/redox-os/pkgutils)  | **@jackpot51**
| [ralloc](https://gitlab.redox-os.org/redox-os/ralloc)                                | **@Tommoa** **@NilSet**
| [RANSID (Rust ANSI driver)](https://gitlab.redox-os.org/redox-os/ransid)             | **@jackpot51**
| [redoxfs (old filesystem)](https://gitlab.redox-os.org/redox-os/redoxfs)             | **@jackpot51**
| [relibc (C Library in Rust)](https://gitlab.redox-os.org/redox-os/relibc)            | **@jD91mZM2** **@sajattack** **@Tommoa** **@stratact**
| [small (stack String and other collections)](https://gitlab.redox-os.org/redox-os/small) | **@Tommoa**
| [syscall](https://gitlab.redox-os.org/redox-os/syscall)                              | **@jackpot51**
| [Sodium (Vim-inspired text editor)](https://gitlab.redox-os.org/redox-os/sodium)     | **vacant**
| [TFS ((ticki) **T**he **F**ile **S**ystem)](https://gitlab.redox-os.org/redox-os/tfs) | **@Tommoa**
| [The Redox book](https://gitlab.redox-os.org/redox-os/book)                          | **vacant**
| [userutils](https://gitlab.redox-os.org/redox-os/userutils)                          | **@jackpot51**

## <a name="compile-help"> Help! Redox won't compile! </a>

Sometimes things go wrong when compiling. Try the following before opening an issue:

1. Run `rustup update`
1. Run `make clean pull`.
1. Make sure you have **the latest version of Rust nightly!** ([rustup.rs](https://www.rustup.rs) is recommended for managing Rust versions. If you already have it, run `rustup`).
1. Update **GNU Make**, **NASM** and **QEMU/VirtualBox**.
1. Pull the upstream master branch (`git remote add upstream git@gitlab.redox-os.org:redox-os/redox.git; git pull upstream master`).
1. Update submodules (`git submodule update --recursive --init`).

and then rebuild!

## <a name="contributing"> Contributing to Redox </a>

If you're interested in this project, and you'd like to help us out, [here](CONTRIBUTING.md) is a list of ways you can do just that.

## <a name="cloning-building-running"> Cloning, Building and Running </a>

Redox is big, even compressed. Downloading the full history may take a lot of bandwidth, and can even be costly on some data plans. Clone at your own risk!

### <a name="quick-setup" /> Quick Setup </a>

```bash
$ cd path/to/your/projects/folder/

# Run bootstrap setup
$ curl -sf https://gitlab.redox-os.org/redox-os/redox/raw/master/bootstrap.sh -o bootstrap.sh && bash -e bootstrap.sh

# Change to project directory
$ cd redox

# Build Redox
$ make all

# Launch using QEMU
$ make qemu
# Launch using QEMU without using KVM (Kernel-based Virtual Machine). Try if QEMU gives an error.
$ make qemu kvm=no
```

#### QEMU with KVM

To use QEMU with KVM (Kernel-based Virtual Machine), which is faster than without KVM, you need a CPU with Intel® Virtualization Technology (Intel® VT) or AMD Virtualization™ (AMD-V™) support. Most systems have this disabled by default, so you may need to reboot, go into the BIOS, and enable it.

### <a name="manual-setup"> Manual Setup </a>

To manually clone, build and run Redox using a Unix-based host, run the following commands (with exceptions, be sure to read the comments):
```bash
$ cd path/to/your/projects/folder/

# HTTPS
$ git clone https://gitlab.redox-os.org/redox-os/redox.git --origin upstream --recursive
# SSH
$ git clone git@gitlab.redox-os.org:redox-os/redox.git --origin upstream --recursive

$ cd redox/

# Install/update dependencies
$ ./bootstrap.sh -d

# Install rustup.rs
$ curl https://sh.rustup.rs -sSf | sh
$ source $HOME/.cargo/env

# Install the sysroot manager Xargo
$ cargo install xargo

# For successive builds start here. If this is your first build, just continue

# Update git submodules
$ git submodule update --recursive --init

# Build Redox
$ make all

# Launch using QEMU
$ make qemu

# Launch using QEMU without using KVM (Kernel-based Virtual Machine). Try if QEMU gives an error.
$ make qemu kvm=no

# Launch using QEMU without using KVM (Kernel-based Virtual Machine) nor Graphics
make qemu kvm=no vga=no
```

### <a name="setup-using-docker"> Setup using Docker </a>
We also provide docker image. After cloning this repository, please follow README under the `docker` directory.

### Updating the codebase using the Makefile
To update the codebase run:

```
make pull; make fetch
```

`make pull` pulls and updates the submodules, and `make fetch` updates the sources for cookbook recipes.
