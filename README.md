<p align="center">
<img alt="Redox" width="346" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/logos/redox/logo.png">
</p>

This repository is the **Build System** for Redox OS.

Redox is under active development by a vibrant community, you can see the key links below:

- [The **main website** for Redox OS](https://www.redox-os.org).
- [The Redox Book](https://doc.redox-os.org/book/) and [Build Instructions](https://doc.redox-os.org/book/podman-build.html).
- [Redox Chat and Support](https://matrix.to/#/#redox-join:matrix.org).
- [Patreon](https://www.patreon.com/redox_os), [Donate](https://redox-os.org/donate/) and [Merch](https://redox-os.creator-spring.com/).
- Scroll down for a list of key Redox components and their repositories.

[Redox](https://www.redox-os.org) is an open-source operating system written in Rust, a language with focus on safety, efficiency and high performance. Redox uses a microkernel architecture, and aims to be reliable, secure, usable, correct, and free. Redox is inspired by previous operating systems, such as seL4, MINIX, Plan 9, Linux and BSD.

Redox _is not_ just a kernel, it's a **full-featured operating system**, providing components (file system, display server, core utilities, etc.) that together make up a functional and convenient operating system. Redox uses the COSMIC desktop apps, and provides source code compatibility with many Rust, Linux and BSD programs.

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## More Links

- [Book](https://doc.redox-os.org/book/)
- [Contribute](CONTRIBUTING.md)
- [Hardware Compatibility](https://doc.redox-os.org/book/hardware-support.html)
- Run Redox in a [Virtual Machine](https://doc.redox-os.org/book/running-vm.html) or on [Real Hardware](https://doc.redox-os.org/book/real-hardware.html)
- [Trying Out Redox](https://doc.redox-os.org/book/trying-out-redox.html)
- [Building Redox](https://doc.redox-os.org/book/podman-build.html)
- [Build System Documentation](https://doc.redox-os.org/book/build-system-reference.html)
- [Developer FAQ](https://doc.redox-os.org/book/developer-faq.html)
- [Chat/Discussions/Help](https://doc.redox-os.org/book/chat.html)

## Ecosystem

Some of the key repositories on the Redox GitLab:

| Essential Repos                                                                      | Maintainer
|--------------------------------------------------------------------------------------|---------------------------
| [Kernel](https://gitlab.redox-os.org/redox-os/kernel)                                | **@jackpot51**
| [RedoxFS (default filesystem)](https://gitlab.redox-os.org/redox-os/redoxfs)         | **@jackpot51**
| [Drivers](https://gitlab.redox-os.org/redox-os/drivers)                              | **@jackpot51**
| [Orbital (display server and window manager)](https://gitlab.redox-os.org/redox-os/orbital) | **@jackpot51**
| [pkgutils (current package manager)](https://gitlab.redox-os.org/redox-os/pkgutils)  | **@jackpot51**
| [relibc (C POSIX library written in Rust)](https://gitlab.redox-os.org/redox-os/relibc)            | **@jackpot51**
| [netstack (network stack)](https://gitlab.redox-os.org/redox-os/netstack)                            | **@jackpot51**
| [Ion (defauilt shell)](https://gitlab.redox-os.org/redox-os/ion)                              | **@jackpot51**
| [Termion (terminal library)](https://gitlab.redox-os.org/redox-os/termion)           | **@jackpot51**
| This repo - the root of the Build System                                             | **@jackpot51**
| [cookbook (Build system for system components and programs)](https://gitlab.redox-os.org/redox-os/cookbook) | **@jackpot51** **@hatred_45**
| [Redoxer (Tool for Redox development testing)](https://gitlab.redox-os.org/redox-os/redoxer) | **@jackpot51**
| [The Redox Book](https://gitlab.redox-os.org/redox-os/book)                          | **@hatred_45**

## What it looks like

See [Redox in Action](https://www.redox-os.org/screens/) for photos and videos.

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/orbital-visual.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-programs.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-term-screenfetch.png">

<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/cosmic-edit-redox.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/website/-/raw/master/static/img/screenshot/image-viewer.png">
<img alt="Redox" height="150" src="https://gitlab.redox-os.org/redox-os/assets/raw/master/screenshots/Boot.png">
