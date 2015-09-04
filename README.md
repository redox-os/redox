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
make clean && make
```

## Running on Ubuntu
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
make clean && make
```

## Running on OS X
- Run Qemu (without network bridge or KVM):
```bash
make run_no_kvm
```

## Building on Windows
- Download and install the latest 32-bit Rust nightly from http://www.rust-lang.org/install.html
- The direct link to the 32-bit nightly is https://static.rust-lang.org/dist/rust-nightly-i686-pc-windows-gnu.msi
- Open the Rust nightly shell
```bash
cd <REDOX REPOSITORY>
windows\make
```
