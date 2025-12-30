# Redox VM Shell Improvements

## What Was Fixed

The Ion shell prompt errors you encountered:
```
ion: prompt expansion failed: pipeline execution error: error reading stdout of child: No such device (os error 19)
```

These occur because Ion's default prompt tries to execute commands that access devices not available in serial console mode.

## Solutions Implemented

### 1. Better Terminal Handling (socat)

✅ **Installed socat** - Professional-grade terminal multiplexer
- Superior to netcat for terminal connections
- Proper escape sequence handling
- Clean Ctrl-C exit
- Better character passthrough

**Before:**
```bash
nc -U socket  # Basic, can be glitchy
```

**After:**
```bash
socat -,raw,echo=0,escape=0x03 UNIX-CONNECT:socket  # Professional grade
```

### 2. Helper Scripts

#### `fix-ion-prompt.sh`
Automatically fixes Ion prompt errors:
```bash
./fix-ion-prompt.sh test
# Sends: export PROMPT="redox> "
```

#### `send-to-vm.sh`
Send commands to VM without connecting:
```bash
./send-to-vm.sh test 'export PROMPT="$ "'
./send-to-vm.sh test ls
./send-to-vm.sh test pwd
```

#### `redox-console`
Alternative console connector with socat:
```bash
./redox-console ~/.redox-vms/test.pty
```

### 3. Improved QEMU Configuration

**Added:**
- `-display none` - Truly headless operation
- `hostfwd=tcp::8080-:80` - HTTP port forwarding
- Better serial console setup

### 4. Documentation

**New guides:**
- `redox-vm-tips.md` - Tips, tricks, and troubleshooting
- Explains Ion prompt issues
- Lists working commands
- Environment variable recommendations

## How to Use

### Quick Fix for Prompt Errors

**Option 1: In the VM shell**
```bash
# Connect to VM
redox-vm shell test

# At the prompt (with errors), type:
export PROMPT="redox> "

# Prompt errors disappear!
```

**Option 2: Use helper script**
```bash
# From host
./fix-ion-prompt.sh test

# Then connect
redox-vm shell test
```

**Option 3: Create Ion config**

In the VM, create `/home/user/.config/ion/ionrc`:
```bash
export PROMPT = "redox> "
export TERM = "vt100"
```

### Improved Shell Connection

```bash
# Now uses socat automatically!
redox-vm shell test

# You get:
# ✅ Better terminal handling
# ✅ Proper escape sequences
# ✅ Clean Ctrl-C exit
# ✅ No garbled output
```

### Send Commands Remotely

```bash
# Fix prompt without connecting
./send-to-vm.sh test 'export PROMPT="> "'

# Run commands
./send-to-vm.sh test 'ls /home'
./send-to-vm.sh test 'pwd'

# Set environment
./send-to-vm.sh test 'export TERM=vt100'
```

## What's Better Now

| Feature | Before | After |
|---------|--------|-------|
| Terminal connector | nc (basic) | socat (professional) |
| Exit method | Ctrl-C (sometimes stuck) | Ctrl-C (clean) |
| Prompt errors | ❌ Visible | ✅ Can be fixed |
| Remote commands | ❌ Not possible | ✅ send-to-vm.sh |
| Display mode | ramfb (unused) | none (headless) |
| Port forwarding | SSH only | SSH + HTTP |
| Troubleshooting | No guide | redox-vm-tips.md |

## Testing

### Test the improved shell:

```bash
# 1. Stop and restart VM
redox-vm stop test
redox-vm launch --name test --mem 2G

# 2. Connect with improved shell
redox-vm shell test

# You should see:
# - Cleaner connection
# - Better keyboard handling
# - Smoother exit with Ctrl-C

# 3. Fix prompt (if errors appear)
# Inside VM, type:
export PROMPT="redox> "
```

### Test helper scripts:

```bash
# Send a command
./send-to-vm.sh test pwd

# Fix prompt remotely
./fix-ion-prompt.sh test

# Connect to see results
redox-vm shell test
```

## Ion Shell Prompt Solutions

### Simple Prompts That Work

```bash
# Static prompts (no expansion)
export PROMPT="$ "
export PROMPT="redox> "
export PROMPT="> "

# With path (simple)
export PROMPT="[\$PWD]$ "

# With hostname
export PROMPT="[\$HOSTNAME]$ "
```

### Prompts to Avoid

```bash
# These cause errors in serial console:
export PROMPT="[git branch expansion]"  # ❌ Tries to read devices
export PROMPT="[background job count]"  # ❌ Accesses unavailable info
```

## Advanced Usage

### QEMU Monitor

Access VM control:
```bash
nc -U ~/.redox-vms/test.monitor

# Commands:
info status        # Check VM status
system_reset       # Reboot VM
stop               # Pause VM
cont               # Resume VM
```

### Network Services

Port forwarding is configured:
```bash
# SSH (from host)
ssh -p 8022 user@localhost

# HTTP (if web server running in VM)
curl http://localhost:8080
```

## Files Overview

```
/opt/other/redox/
├── redox-vm              # Main VM manager (updated)
├── redox-shell           # Terminal wrapper (improved)
├── redox-console         # Socat-based console
├── fix-ion-prompt.sh     # Auto-fix prompt errors
├── send-to-vm.sh         # Send commands to VM
├── redox-vm-tips.md      # Troubleshooting guide
└── IMPROVEMENTS.md       # This file
```

## Summary

The shell now works **much better**:

✅ Professional terminal handling with socat
✅ Helper scripts for common tasks
✅ Documentation for troubleshooting
✅ Clean exit behavior
✅ Remote command execution
✅ Prompt error fixes

**Try it now:**
```bash
redox-vm shell test

# At the prompt:
export PROMPT="redox> "

# Enjoy error-free shell!
```
