<p align="center">
<img alt="Redox" width="346" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/logos/redox/logo.png">
</p>

**Redox** is an operating system written in Rust, a language with focus on safety and high performance. Redox, following the microkernel design, aims to be secure, usable, and free. Redox is inspired by previous kernels and operating systems, such as SeL4, MINIX, Plan 9, and BSD.

Redox _is not_ just a kernel, it's a **full-featured Operating System**, providing packages (memory allocator, file system, display manager, core utilities, etc.) that together make up a functional and convenient operating system. You can loosely think of it as the GNU or BSD ecosystem, but in a memory safe language and with modern technology.

The website can be found at https://www.redox-os.org. For hardware compatibility, see [HARDWARE.md](HARDWARE.md).

[![Downloads](https://img.shields.io/github/downloads/redox-os/redox/total.svg)](https://gitlab.redox-os.org/redox-os/redox/tags)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## What it looks like

See [Redox in Action](https://www.redox-os.org/screens/).

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Senza%20titolo.jpeg">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/redox running.jpeg">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/IMG_1460.PNG">

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Sodium_v2.PNG">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Boot.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/IMG_1459.PNG">


## Getting Started

Please read the [Getting Started Guide](https://doc.redox-os.org/book/ch02-00-getting-started.html) in the [Redox Book](https://doc.redox-os.org/book/).

### Trying Out Redox

You can try out the Redox download in an emulator, or boot from a USB. Please see the [Getting Started Guide](https://doc.redox-os.org/book/ch02-00-getting-started.html).

### Building Redox

The Redox build system is unique and complex, so you are best to follow the practices in the book. Please see the [Getting Started Guide](https://doc.redox-os.org/book/ch02-00-getting-started.html).

### Getting Help

Please join us on [Redox Chat](https://doc.redox-os.org/book/ch13-01-chat.html) for any concerns. You can post your questions or issues in [Redox Support](https://matrix.to/#/#redox-support:matrix.org) room.

## Contributing

If you're interested in this project, and you'd like to help us out, have a look at [CONTRIBUTING](CONTRIBUTING.md).

## Ecosystem

The ecosystem and software Redox OS provides is listed below.

| Name (lexicographic order)                                                           | Maintainer
|--------------------------------------------------------------------------------------|---------------------------
| [acid (kernel integration tests)](https://gitlab.redox-os.org/redox-os/acid)         | **@jackpot51**
| [binutils](https://gitlab.redox-os.org/redox-os/binutils)                            | **@jackpot51**
| [cookbook](https://gitlab.redox-os.org/redox-os/cookbook)                            | **@jackpot51** **@hatred_45** **@ids1024**
| [coreutils](https://gitlab.redox-os.org/redox-os/coreutils)                          | **@jackpot51**
| [extrautils](https://gitlab.redox-os.org/redox-os/extrautils)                        | **@jackpot51**
| [games](https://gitlab.redox-os.org/redox-os/games)                                  | **@fabiao**
| [Ion (shell)](https://gitlab.redox-os.org/redox-os/ion)                              | **@jackpot51**
| [ipcd](https://gitlab.redox-os.org/redox-os/ipcd)                                    | **@jackpot51**
| [kernel](https://gitlab.redox-os.org/redox-os/kernel)                                | **@jackpot51**
| [libextra](https://gitlab.redox-os.org/redox-os/libextra)                            | **@jackpot51**
| [libpager](https://gitlab.redox-os.org/redox-os/libpager)                            | **@jackpot51**
| [netstack](https://gitlab.redox-os.org/redox-os/netstack)                            | **@jackpot51**
| [netutils](https://gitlab.redox-os.org/redox-os/netutils)                            | **@jackpot51**
| [orbclient (Orbital client)](https://gitlab.redox-os.org/redox-os/orbclient)         | **@jackpot51** **@FloVanGH**
| [orbdata](https://gitlab.redox-os.org/redox-os/orbdata)                              | **@jackpot51**
| [orbgame (Orbital 2D game engine)](https://gitlab.redox-os.org/redox-os/orbgame)     | **@FloVanGH**
| [Orbital (windowing and compositing system)](https://gitlab.redox-os.org/redox-os/orbital) | **@jackpot51**
| [orbtk (Orbital toolkit)](https://gitlab.redox-os.org/redox-os/orbtk)                | **@FloVanGH**
| [orbutils (Orbital utilities)](https://gitlab.redox-os.org/redox-os/orbutils)        | **@jackpot51**
| [pkgutils (current package manager)](https://gitlab.redox-os.org/redox-os/pkgutils)  | **@jackpot51**
| [ralloc](https://gitlab.redox-os.org/redox-os/ralloc)                                | **@jackpot51**
| [RANSID (Rust ANSI driver)](https://gitlab.redox-os.org/redox-os/ransid)             | **@jackpot51**
| [redoxfs (default filesystem)](https://gitlab.redox-os.org/redox-os/redoxfs)             | **@jackpot51**
| [relibc (C Library in Rust)](https://gitlab.redox-os.org/redox-os/relibc)            | **@jackpot51**
| [small (stack String and other collections)](https://gitlab.redox-os.org/redox-os/small) | **@jackpot51**
| [syscall](https://gitlab.redox-os.org/redox-os/syscall)                              | **@jackpot51**
| [Sodium (Vim-inspired text editor)](https://gitlab.redox-os.org/redox-os/sodium)     | **@jackpot51**
| [The Redox book](https://gitlab.redox-os.org/redox-os/book)                          |    **@hatred_45**
| [userutils](https://gitlab.redox-os.org/redox-os/userutils)                          | **@jackpot51**

