#!/bin/bash
# Quick fix for currently connected session

VM_NAME="${1:-test}"
PTY_SOCKET="$HOME/.redox-vms/${VM_NAME}.pty"

if [[ ! -S "$PTY_SOCKET" ]]; then
    echo "Error: VM '$VM_NAME' not running"
    exit 1
fi

echo "Sending quick fixes to $VM_NAME..."

# Send the fix commands
{
    sleep 0.5
    # Fix prompt
    printf 'export PROMPT="redox> "\r\n'
    sleep 0.3
    # Fix localization
    printf 'export LC_ALL="C"\r\n'
    sleep 0.3
    printf 'export LANG="C"\r\n'
    sleep 0.3
    # Confirm
    printf 'echo "Fixes applied! Prompt and localization errors should be gone."\r\n'
    sleep 0.5
} | nc -U "$PTY_SOCKET"

echo ""
echo "Fixes sent! The errors should disappear now."
echo "Connect to verify:"
echo "  redox-vm shell $VM_NAME"
