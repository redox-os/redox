#!/bin/bash
# Fix Ion shell prompt errors in Redox VM
# This script sends commands to fix the prompt via the serial console

VM_NAME="${1:-test}"
PTY_SOCKET="$HOME/.redox-vms/${VM_NAME}.pty"

if [[ ! -S "$PTY_SOCKET" ]]; then
    echo "Error: VM '$VM_NAME' not running or socket not found"
    echo "Usage: $0 [vm-name]"
    exit 1
fi

echo "Fixing Ion prompt for VM: $VM_NAME"
echo "Sending commands to set simple prompt..."
echo ""

# Send commands to fix prompt
# Using printf with \r\n for proper line endings
{
    sleep 1
    printf 'export PROMPT="redox> "\r\n'
    sleep 0.5
    printf 'echo "Prompt fixed!"\r\n'
    sleep 0.5
} | nc -U "$PTY_SOCKET" &

echo "Commands sent. Connect to VM to verify:"
echo "  redox-vm shell $VM_NAME"
echo ""
echo "The prompt should now be: redox>"
