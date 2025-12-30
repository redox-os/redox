#!/bin/bash
# Send commands to running Redox VM and capture output

PTY="/Users/me/.redox-vms/test.pty"

if [[ ! -S "$PTY" ]]; then
    echo "VM not running or PTY not available"
    exit 1
fi

# Function to send command and wait
send_cmd() {
    local cmd="$1"
    echo "$cmd" | socat - "UNIX-CONNECT:$PTY"
    sleep 2
}

echo "=== Sending commands to Redox OS ==="

send_cmd "env"
send_cmd "ls -la ~"
send_cmd "pwd"
send_cmd "uname -a"

echo "=== Commands sent, check VM logs ==="
