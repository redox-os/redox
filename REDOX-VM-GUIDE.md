# Redox VM Manager Guide

A multipass-like interface for managing Redox OS virtual machines.

## Quick Start

```bash
# Launch a VM (similar to multipass)
./redox-vm launch --mem 4G --name redox-me --mount $HOME:/Users/me

# List VMs
./redox-vm list

# Shell into VM (when supported)
./redox-vm shell redox-me

# Stop VM
./redox-vm stop redox-me

# Delete VM
./redox-vm delete redox-me
```

## Installation

### Prerequisites

1. **QEMU with aarch64 support**:
   ```bash
   brew install qemu
   ```

2. **Build tools** (if building from source):
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install build dependencies
   brew install coreutils findutils gcc nasm make
   ```

### Optional: Add to PATH

For convenience, you can create a symlink:

```bash
# Add to /usr/local/bin
sudo ln -s /opt/other/redox/redox-vm /usr/local/bin/redox-vm

# Or add to your shell profile
echo 'export PATH="/opt/other/redox:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Building Redox Images

The first time you launch a VM, Redox will build the image automatically. This can take 30-60 minutes.

To build manually:

```bash
# Build desktop image for aarch64
make ARCH=aarch64 CONFIG_NAME=desktop all

# Or use the build script
./build.sh
```

## VM Configurations

Set environment variables to customize:

```bash
# Use minimal configuration
CONFIG_NAME=minimal ./redox-vm launch --name redox-minimal --mem 2G

# Available configs: desktop, minimal, server, dev
```

## Architecture

Redox VM Manager stores VM data in `~/.redox-vms/`:

- `<name>.info` - VM metadata (memory, PID, etc.)
- `<name>.img` - VM disk image (copy of built image)
- `<name>.log` - VM console output

## Current Limitations

1. **Interactive shell**: Not yet supported. Access via serial console in logs.
2. **Directory mounting**: Planned feature, not implemented yet.
3. **Snapshots**: Not implemented.
4. **SSH**: Network is configured but SSH access requires setup in guest OS.

## Troubleshooting

### QEMU fails to start

Check the logs:
```bash
tail -f ~/.redox-vms/<name>.log
```

### UEFI firmware not found

Ensure QEMU is properly installed:
```bash
brew reinstall qemu
```

### Build fails

Check you have all dependencies:
```bash
cd /opt/other/redox
make env
```

## Advanced Usage

### Custom QEMU flags

Edit the `redox-vm` script to customize QEMU parameters in the `cmd_launch()` function.

### Using different architectures

```bash
ARCH=x86_64 ./redox-vm launch --name redox-x64 --mem 4G
```

## Comparison with Multipass

| Feature | Multipass | Redox VM | Status |
|---------|-----------|----------|--------|
| Launch VMs | ✅ | ✅ | Working |
| Shell access | ✅ | ⚠️ | Serial console only |
| Directory mounting | ✅ | ❌ | Planned |
| List VMs | ✅ | ✅ | Working |
| Stop/Delete VMs | ✅ | ✅ | Working |
| Snapshots | ✅ | ❌ | Not planned |
| Cloud-init | ✅ | ❌ | N/A |

## Contributing

Redox OS is an open source project. Contributions welcome!

- Main repo: https://gitlab.redox-os.org/redox-os/redox
- Documentation: https://doc.redox-os.org/book/
