#!/bin/bash
# Send Enter key to Redox VM to proceed past bootloader

VM_NAME="test"
PID=$(cat ~/.redox-vms/${VM_NAME}.info 2>/dev/null | grep "^PID=" | cut -d= -f2)

if [ -z "$PID" ]; then
    echo "VM not found or not running"
    exit 1
fi

# Send Enter key to proceed
echo "" | nc -U /tmp/redox-${VM_NAME}-monitor 2>/dev/null || echo "Monitor not available"

# Alternative: send signal to continue
kill -USR1 $PID 2>/dev/null || true

echo "Sent boot signal to VM (PID: $PID)"
echo "Check logs: tail -f ~/.redox-vms/${VM_NAME}.log"
