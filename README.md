# Redox
Redox is a Rust based operating system, designed to be modular and well documented (TODO).

## Building on Ubuntu
- Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./ubuntu.sh
./binary.sh
```
- Make the project
```bash
make
```

## Running on Ubuntu
- Install Qemu
```bash
sudo apt-get install qemu-system-x86 qemu-kvm uml-utilities
```
- Run Qemu (without network bridge):
```bash
make run
```
- Run Qemu (with network bridge, requires sudo password, guest accessible at 10.85.85.2):
```bash
make run_tap
```

## Building on OS X
- Install MacPorts
- Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./osx.sh
./binary.sh
```
- Make the project
```bash
make
```

## Running on OS X
- Install VirtualBox from https://www.virtualbox.org/wiki/Downloads
- Run Qemu (without network bridge or KVM):
```bash
make run_virtualbox
```

## Building on Windows
- Download and install the latest 32-bit Rust nightly from http://www.rust-lang.org/install.html
- The direct link to the 32-bit nightly is https://static.rust-lang.org/dist/rust-nightly-i686-pc-windows-gnu.msi
- Open the Rust nightly shell
```bash
cd <REDOX REPOSITORY>
windows\make
```

## Running on Windows
- Install VirtualBox from https://www.virtualbox.org/wiki/Downloads
- Make sure to install to C:\Program Files\Oracle\VirtualBox or edit the Makefile VBM path
- Run Virtualbox (without network bridge or KVM):
```bash
windows\make run_virtualbox
```
