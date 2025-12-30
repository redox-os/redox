#!/bin/bash
# Test redox-vm shell command

echo "=== Testing Redox VM Shell ==="
echo ""

# Check if test VM is running
if ! redox-vm list | grep -q "test.*Running"; then
    echo "Starting test VM..."
    redox-vm launch --name test --mem 2G
    sleep 5
fi

echo "VM Status:"
redox-vm list | grep test

echo ""
echo "PTY Socket:"
ls -l ~/.redox-vms/test.pty

echo ""
echo "To connect to the VM shell, run:"
echo "  redox-vm shell test"
echo ""
echo "Inside the VM, you can:"
echo "  - Login (if configured)"
echo "  - Run commands"
echo "  - Press Ctrl-C to exit"
echo ""
echo "VM console output:"
tail -20 ~/.redox-vms/test.log
