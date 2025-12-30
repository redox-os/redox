# Redox VM - Multipass-like VM Manager for Redox OS

## Overview

`redox-vm` is a command-line tool that provides a multipass-like interface for managing Redox OS virtual machines on macOS (Apple Silicon).

## Installation

The tool is already installed and available globally as `redox-vm`.

### Prerequisites

```bash
# QEMU should already be installed, if not:
brew install qemu
```

## Quick Start

```bash
# Launch a VM (builds image on first run - takes 30+ min)
redox-vm launch --mem 4G --name redox-me --mount $HOME:/Users/me

# List all VMs
redox-vm list

# View VM console output
tail -f ~/.redox-vms/redox-me.log

# Stop a VM
redox-vm stop redox-me

# Delete a VM
redox-vm delete redox-me
```

## Commands

### Launch a VM

```bash
redox-vm launch --mem <size> --name <name> [--mount <src:dst>]

# Examples:
redox-vm launch --name my-redox --mem 4G
CONFIG_NAME=minimal redox-vm launch --name small-vm --mem 2G
```

Options:
- `--mem` - Memory size (e.g., 2G, 4096M)
- `--name` - VM name (required)
- `--mount` - Host directory mount (planned, not yet implemented)

### List VMs

```bash
redox-vm list
```

Shows all VMs with their status, memory, architecture, and configuration.

### Shell into VM

```bash
redox-vm shell <name>
```

⚠️ **Note**: Interactive shell not yet fully supported. Access via serial console in logs for now.

### Stop a VM

```bash
redox-vm stop <name>
```

### Delete a VM

```bash
redox-vm delete <name>
```

Stops the VM and removes all associated files (disk image, logs, metadata).

## Environment Variables

Customize VM creation with environment variables:

```bash
# Architecture (default: aarch64)
ARCH=aarch64 redox-vm launch --name test --mem 4G

# Configuration (default: desktop)
# Options: desktop, minimal, server, dev
CONFIG_NAME=minimal redox-vm launch --name test --mem 2G
```

## Directory Structure

VMs are stored in `~/.redox-vms/`:
```
~/.redox-vms/
├── <name>.info    # VM metadata
├── <name>.img     # Disk image
└── <name>.log     # Console output
```

## Building Images

On first launch, Redox will build an OS image. This takes 30-60 minutes.

To pre-build manually:
```bash
cd /opt/other/redox
make ARCH=aarch64 CONFIG_NAME=desktop all
```

## Features Comparison

| Feature | Status | Notes |
|---------|--------|-------|
| Launch VMs | ✅ | Working |
| List VMs | ✅ | Working |
| Stop VMs | ✅ | Working |
| Delete VMs | ✅ | Working |
| Shell access | ⚠️ | Via serial console in logs |
| Directory mounting | ❌ | Planned |
| Multiple architectures | ✅ | aarch64, x86_64, i586, riscv64gc |
| Custom configs | ✅ | desktop, minimal, server, dev |

## Troubleshooting

### Check VM logs
```bash
tail -f ~/.redox-vms/<name>.log
```

### QEMU not starting
Ensure QEMU is installed:
```bash
brew list qemu
brew reinstall qemu
```

### Build failures
Check build dependencies:
```bash
cd /opt/other/redox
make env
```

## Advanced Usage

### Custom configurations

Available configurations in `config/aarch64/`:
- `desktop.toml` - Full desktop environment
- `minimal.toml` - Minimal system
- `server.toml` - Server configuration
- `dev.toml` - Development environment

### Using make directly

For more control, use the Makefile:
```bash
cd /opt/other/redox

# Build
make ARCH=aarch64 CONFIG_NAME=desktop all

# Run with QEMU
make ARCH=aarch64 CONFIG_NAME=desktop qemu
```

## Implementation

- **Script**: `/opt/other/redox/redox-vm`
- **Wrapper**: `/usr/local/bin/redox-vm`
- **VM Storage**: `~/.redox-vms/`
- **VM Manager**: Uses QEMU with HVF acceleration (Apple Silicon)

## See Also

- [Full Guide](REDOX-VM-GUIDE.md)
- [Redox Documentation](https://doc.redox-os.org/book/)
- [Redox GitLab](https://gitlab.redox-os.org/redox-os/redox)
