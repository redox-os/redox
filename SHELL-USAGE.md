# Redox VM Shell Usage Guide

## Interactive Shell Access

The `redox-vm shell` command provides direct serial console access to running Redox VMs.

## Basic Usage

```bash
# Launch a VM
redox-vm launch --name myvm --mem 4G

# Connect to the shell
redox-vm shell myvm
```

## How It Works

### Architecture

```
User Terminal → redox-shell → UNIX Socket → QEMU Serial Console → Redox OS
```

1. **QEMU Configuration**: VM serial port connected to UNIX socket
2. **Socket Path**: `~/.redox-vms/<vm-name>.pty`
3. **Connection**: netcat (nc) provides bidirectional I/O
4. **Terminal**: Raw mode for direct character passthrough

### Technical Details

- **Serial Device**: UNIX socket (server mode, non-blocking)
- **Terminal Mode**: Raw input, echo disabled
- **Exit Method**: Ctrl-C (SIGINT)
- **Socket Type**: SOCK_STREAM UNIX domain socket

## Usage Examples

### Connect to a Running VM

```bash
$ redox-vm shell myvm
[redox-vm] Connecting to myvm serial console...
[redox-vm] Press Ctrl-C to exit

# You're now in the Redox OS console
# Boot messages will appear
# Login prompt (if configured)
```

### Check VM Status First

```bash
$ redox-vm list
Name            State      Memory   Arch      Config
--------------------------------------------------------
myvm            Running    4G       aarch64   desktop

$ redox-vm shell myvm
```

### Restart VM with New Shell Support

If VM was launched before shell support:

```bash
# Stop old VM
redox-vm stop myvm

# Relaunch (will use existing disk image)
redox-vm launch --name myvm --mem 4G

# Connect
redox-vm shell myvm
```

## Troubleshooting

### Socket Not Found

```bash
$ redox-vm shell myvm
[redox-vm] Serial console socket not found: /Users/me/.redox-vms/myvm.pty
[redox-vm] VM may have been started with old configuration
[redox-vm] Try: redox-vm stop myvm && redox-vm launch --name myvm --mem 4G
```

**Solution**: Restart the VM to create the socket.

### VM Not Running

```bash
$ redox-vm shell myvm
[redox-vm] VM 'myvm' is not running
```

**Solution**: Start the VM first with `redox-vm launch`.

### Netcat Not Available

```bash
$ redox-vm shell myvm
[redox-vm] netcat (nc) not found. Cannot connect to serial console.
[redox-vm] Install netcat or socat:
[redox-vm]   brew install netcat
[redox-vm]   brew install socat
```

**Solution**: Install netcat (usually pre-installed on macOS).

## Terminal Behavior

### Input/Output

- **Characters**: Passed directly to VM (no buffering)
- **Special Keys**: Function keys, arrow keys work
- **Colors**: ANSI color codes supported
- **Backspace**: Works as expected
- **Tab**: Command completion (if supported by shell)

### Exiting the Console

**Press Ctrl-C** to disconnect from the console.

The VM continues running after disconnect.

### Terminal Restoration

On exit, the terminal is automatically restored to normal mode:
- Echo re-enabled
- Cooked mode restored
- Cursor visible

## Advanced Usage

### Direct Socket Access

```bash
# Using netcat directly
nc -U ~/.redox-vms/myvm.pty

# Using socat with better terminal handling
socat -,raw,echo=0,escape=0x11 UNIX-CONNECT:~/.redox-vms/myvm.pty
```

### Monitor Socket

QEMU monitor also available:

```bash
# Access QEMU monitor
nc -U ~/.redox-vms/myvm.monitor

# Monitor commands
info status
info network
quit
```

### Multiple Connections

⚠️ **Warning**: Multiple simultaneous connections to the same PTY socket may cause issues. Only one active shell session per VM is recommended.

## Comparison with SSH

| Feature | Serial Console | SSH |
|---------|----------------|-----|
| Boot messages | ✅ Visible | ❌ Not available |
| Early access | ✅ From BIOS/UEFI | ❌ After network |
| Network required | ❌ No | ✅ Yes |
| Setup required | ❌ No | ✅ SSH server |
| Multiple sessions | ⚠️ Limited | ✅ Multiple |
| File transfer | ❌ No | ✅ SCP/SFTP |

## Best Practices

1. **Use for debugging**: Serial console shows all boot messages
2. **Check logs first**: `tail -f ~/.redox-vms/<name>.log` for persistent output
3. **One session**: Avoid multiple simultaneous shell connections
4. **Exit cleanly**: Use Ctrl-C to disconnect properly
5. **VM lifecycle**: Shell survives VM reboots (reconnects automatically)

## Future Enhancements

Planned features:
- SSH support (port 8022 already forwarded)
- Automatic reconnection on disconnect
- Session recording
- Multiple terminal support
- Copy/paste integration

## See Also

- [README-REDOX-VM.md](README-REDOX-VM.md) - Main documentation
- [REDOX-VM-GUIDE.md](REDOX-VM-GUIDE.md) - Full guide
- `redox-vm --help` - Command reference
