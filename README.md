<img alt="Redox" height="150" src="img/logo.png">

**Redox** is a operating system written in pure Rust, designed to be modular and secure. The development blog can be found at http://www.redox-os.org.

Documentation can be found [here](http://ticki.github.io/redocs/redox/).

Please make sure you use the **latest nightly** of `rustc` before building (for more troubleshooting, see ["Help! Redox won't compile!"](#compile-help)).

![travis](https://api.travis-ci.org/redox-os/redox.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)

## Contents

* [What it looks like](#what-it-looks-like)
* [Help! Redox won't compile](#compile-help)
* [Contributing to Redox](#contributing)
* [Cloning the repository](#cloning)
* [Installation](#installation)
  * [Building and running](#bulding-running)
    * [Debian and Ubuntu family](#debian-ubuntu)
    * [Archlinux](#arch-linux)
    * [Fedora](#fedora)
    * [Suse](#suse)
    * [NixOS](#nixos)
    * [OS X](#osx)
    * [Windows](#windows)


## <a name="what-it-looks-like" />What it looks like

<img alt="Redox" height="150" src="img/screenshots/Desktop.png">
<img alt="Redox" height="150" src="img/screenshots/Fancy_opacity.png">
<img alt="Redox" height="150" src="img/screenshots/File_manager.png">
<img alt="Redox" height="150" src="img/screenshots/Sodium_v1.png">
<img alt="Redox" height="150" src="img/screenshots/Boot.png">
<img alt="Redox" height="150" src="img/screenshots/start.png">

## <a name="compile-help" />Help! Redox won't compile!

Sometimes things go wrong when compiling. Try the following before opening an issue:

1. Run `make clean`.
2. Run `git clean -X -f -d`.
3. Make sure you have **the latest version of Rust nightly!** ([multirust](https://github.com/brson/multirust) is recommended for managing Rust versions).
4. Update **LLVM**, **GNU Make**, **nasm** and **QEMU/VirtualBox**.
5. Pull the upstream master branch (`git remote add upstream git@github.com:redox-os/redox.git; git pull upstream master`).

and then rebuild!

## <a name="contributing" />Contributing to Redox
If you're interested in this project, and you'd like to help us out, [here](CONTRIBUTING.md) is a list of ways you can do just that.

## <a name="cloning" />Cloning the Repository

Make sure you get submodules when you clone the repository.
```bash
git clone --recursive
```

If you already have a copy of the repository locally without submodules, you
can download them with:
```bash
git submodule update --init
```

## <a name="installation" />Installation

### <a name="building-running" />Building and running

#### <a name="debian_ubuntu" />Debian/Ubuntu family

##### Building

* Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./ubuntu.sh
./binary.sh
```
* Make the project
```bash
make all
```

##### Running

* Install VirtualBox
```bash
sudo apt-get install virtualbox
```
* Run VirtualBox
```bash
make virtualbox
```

##### Running (Qemu, Advanced)
* Install Qemu
```bash
sudo apt-get install qemu-system-x86 qemu-kvm
```
* Run Qemu
```bash
make qemu
```

#### <a name="arch-linux"></a>Arch Linux

##### Building
* Run the setup script and enter your password when prompted (to install the Rust compiler and its dependencies)
```bash
cd setup
./arch.sh
./binary.sh
```
* Make the project
```bash
make
```

##### Running

* Virtualbox was completely setup as part of the script.
* Run Virtualbox
```bash
make virtualbox
```

##### Running (Qemu, Advanced)

* Install Qemu
```bash
$ sudo pacman -S qemu
```
* Run redox
```bash
$ make qemu
```

#### <a name="fedora"></a>Fedora

##### Building

* Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./fedora.sh
./binary.sh
```
* Make the project
```bash
make all
```

##### Running (Qemu, Advanced)

* Install Qemu
```bash
sudo yum install qemu-system-x86 qemu-kvm
```
* Run Qemu
```bash
make qemu
```

#### <a name="suse" />SUSE

##### Building

* Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./suse.sh
./binary.sh
```
* Make the project
```bash
make all
```

##### Running (Qemu, Advanced)

* Install Qemu
```bash
sudo zypper install qemu-x86 qemu-kvm
```
* Run Qemu
```bash
make qemu
```

#### <a name="nixos" />NixOS

##### Building and running (Qemu, Advanced)

```bash
nix-shell setup/dev-env.nix
make all
make qemu
```

#### <a name="osx" />OS X

##### Building

* Install MacPorts or Homebrew
* Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
# MacPorts
./osx-macports.sh
# Homebrew
./osx-homebrew.sh
./binary.sh
```
* Make the project
```bash
make all
```

##### Running
* Install VirtualBox from https://www.virtualbox.org/wiki/Downloads
* Make sure it is installed for all users, in /Applications/ or edit the Makefile VBM path
* Run VirtualBox
```bash
make virtualbox
```

#### <a name="windows" />Windows

##### Building
* Download and install the latest 32-bit Rust nightly from http://www.rust-lang.org/install.html
* The direct link to the 32-bit nightly is https://static.rust-lang.org/dist/rust-nightly-i686-pc-windows-gnu.msi
* Open the Rust nightly shell in the redox repository
```bash
make all
```

##### Running
* Install VirtualBox from https://www.virtualbox.org/wiki/Downloads
* Make sure to install to C:\Program Files\Oracle\VirtualBox or edit the Makefile VBM path
* Run VirtualBox
```bash
make virtualbox
```
