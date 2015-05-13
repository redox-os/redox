# Redox
Redox is a Rust based operating system, designed to be modular and well documented (TODO).

## Building on Ubuntu
- Run the setup script and enter your password when prompted (to install Rust compiler and its dependencies)
```bash
cd setup
./ubuntu.sh
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
