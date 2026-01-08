#!/bin/bash
# Test 9p write functionality in Redox OS
# Run this after booting Redox with run-backup.sh or run-debug.sh

echo "=== 9p Write Test ==="
echo "This script tests writing to 9p share from Redox"
echo ""
echo "Prerequisites:"
echo "1. Redox booted with 9p share enabled"
echo "2. Connect to serial console"
echo "3. Run these commands in Redox:"
echo ""
echo "# Test reading (should already work):"
echo "cat /scheme/9p.hostshare/host-file.txt"
echo ""
echo "# Test writing:"
echo "echo 'Hello from Redox' > /scheme/9p.hostshare/redox-write.txt"
echo ""
echo "# Create a file:"
echo "touch /scheme/9p.hostshare/new-file.txt"
echo ""
echo "# Then check on host:"
echo "cat /tmp/9p-share/redox-write.txt"
echo ""

# Create test file on host
mkdir -p /tmp/9p-share
echo "This file is from the host at $(date)" > /tmp/9p-share/host-file.txt
echo "Created test file: /tmp/9p-share/host-file.txt"
