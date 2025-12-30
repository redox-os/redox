#!/bin/bash
# Create Ion configuration to fix prompt and localization errors

VM_NAME="${1:-test}"
PTY_SOCKET="$HOME/.redox-vms/${VM_NAME}.pty"

if [[ ! -S "$PTY_SOCKET" ]]; then
    echo "Error: VM '$VM_NAME' not running"
    echo "Usage: $0 [vm-name]"
    exit 1
fi

echo "Creating Ion configuration for VM: $VM_NAME"
echo "This will fix:"
echo "  1. Ion prompt expansion errors"
echo "  2. Localization system errors"
echo ""

# Create the commands to send
cat << 'EOF' | nc -U "$PTY_SOCKET" -w 1

# Create config directory
mkdir -p /home/user/.config/ion

# Create ionrc with fixes
cat > /home/user/.config/ion/ionrc << 'IONRC'
# Ion Shell Configuration
# Fixes for serial console

# Simple prompt (no device access)
export PROMPT = "redox> "

# Terminal settings
export TERM = "vt100"

# Disable localization to prevent errors
export LC_ALL = "C"
export LANG = "C"

# Welcome message
echo "Ion shell configured for serial console"
IONRC

# Apply settings immediately
export PROMPT="redox> "
export LC_ALL="C"
export LANG="C"

echo "Configuration created and applied!"

EOF

sleep 2

echo ""
echo "Configuration sent to VM!"
echo "Now reconnect to apply:"
echo "  redox-vm shell $VM_NAME"
echo ""
echo "Or the fixes will apply automatically on next login"
