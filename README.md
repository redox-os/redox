<img alt="Redox" height="150" src="img/logo.png">

**Redox** is an operating system written in pure Rust, designed to be modular and secure. The development blog can be found at http://www.redox-os.org.

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
