#!/bin/bash
# Send commands to Redox VM serial console

VM_NAME="${1}"
shift
COMMAND="$*"

if [[ -z "$VM_NAME" ]] || [[ -z "$COMMAND" ]]; then
    echo "Usage: $0 <vm-name> <command>"
    echo ""
    echo "Examples:"
    echo "  $0 test ls"
    echo "  $0 test 'export PROMPT=\"> \"'"
    echo "  $0 test pwd"
    exit 1
fi

PTY_SOCKET="$HOME/.redox-vms/${VM_NAME}.pty"

if [[ ! -S "$PTY_SOCKET" ]]; then
    echo "Error: VM '$VM_NAME' not running or socket not found"
    exit 1
fi

echo "Sending to $VM_NAME: $COMMAND"

# Send command with proper line ending
printf '%s\r\n' "$COMMAND" | nc -U "$PTY_SOCKET" -w 1

echo "Command sent. Connect to see output:"
echo "  redox-vm shell $VM_NAME"
