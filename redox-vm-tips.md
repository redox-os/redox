# Redox VM Tips & Tricks

## Fixing Ion Shell Prompt Errors

The error you're seeing:
```
ion: prompt expansion failed: pipeline execution error: error reading stdout of child: No such device (os error 19)
```

This happens because Ion shell's default prompt tries to access devices not available in serial console mode.

### Solution 1: Set a Simple Prompt

In the Redox VM:

```bash
# Set a simple prompt without fancy expansions
export PROMPT="redox> "
```

Or for a more informative prompt:
```bash
export PROMPT="[\$PWD]$ "
```

### Solution 2: Disable Prompt Expansion

```bash
# Use a static prompt
export PROMPT='$ '
```

### Solution 3: Create a .ionrc Config

Create `/home/user/.config/ion/ionrc`:
```bash
# Simple prompt that works in serial console
export PROMPT = "redox> "
```

## Working Commands in Serial Console

These work well:
```bash
# File operations
ls
cd /
pwd
cat file.txt

# Process management
ps
top
kill <pid>

# System info
uname -a
free
df

# Network (if configured)
ip addr
ping 8.8.8.8

# Help
help
man <command>
```

## Commands to Avoid

Some commands don't work well in serial console:
- Commands requiring PTY (pseudo-terminal)
- Interactive editors expecting terminal control
- Programs using ncurses/terminfo heavily

## Better Terminal Experience

### Install socat (Done!)

Already installed. This gives you:
- Better character handling
- Proper escape sequences
- Cleaner exit (Ctrl-C)

### Set TERM Variable

In Redox:
```bash
export TERM=vt100
# or
export TERM=xterm
```

## Useful Redox Commands

```bash
# See running services
ls /scheme/

# Check memory
free

# Network info
ip addr
ip link

# Disk usage
df -h

# Package management (if available)
pkg list
pkg install <package>

# Edit text files
# Note: vi/vim may not work well in serial console
# Use cat/echo for simple edits
echo "text" > file.txt
cat >> file.txt << EOF
line 1
line 2
EOF
```

## Accessing Files from Host

### Via Network (if SSH is configured)

```bash
# On host
scp -P 8022 file.txt user@localhost:/home/user/

# Or use redoxfs mounting (advanced)
```

## Troubleshooting

### Garbled Output

Press Ctrl-C to exit and reconnect:
```bash
redox-vm shell myvm
```

### Stuck Terminal

1. Exit with Ctrl-C
2. Reset terminal: `reset` or `stty sane`
3. Reconnect

### Can't Type

Terminal might be in wrong mode:
```bash
# On host, before reconnecting
stty sane
redox-vm shell myvm
```

## Advanced: QEMU Monitor

Access QEMU monitor for VM control:
```bash
# In another terminal
nc -U ~/.redox-vms/test.monitor

# Monitor commands
info status        # VM status
info network       # Network info
stop               # Pause VM
cont               # Resume VM
system_reset       # Reset VM
quit               # Shutdown VM
```

## Environment Variables

Useful exports for Redox shell:
```bash
export TERM=vt100
export PS1="redox> "
export EDITOR=nano  # or vi if available
export PAGER=less
```

## Creating a Better Profile

Create `/home/user/.profile`:
```bash
#!/bin/ion

# Set simple prompt
export PROMPT = "[\$PWD]$ "

# Set terminal
export TERM = "vt100"

# Aliases
alias ll = "ls -la"
alias .. = "cd .."

# Welcome message
echo "Welcome to Redox OS!"
echo "Type 'help' for available commands"
```

## Performance Tips

1. **Use minimal config** for VMs you'll use via serial console
2. **Disable graphical services** not needed
3. **Allocate appropriate memory** (2G usually enough for console work)

## Next Steps

Once you're comfortable with serial console:
1. Set up SSH server in Redox
2. Connect via SSH (port 8022 forwarded)
3. Use full terminal features (scp, sftp, etc.)
