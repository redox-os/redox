#!/usr/bin/env bash
# Inject Ion configuration into Redox VM disk image
set -e

REDOX_ROOT="$(cd "$(dirname "$0")" && pwd)"
REDOXFS="${REDOX_ROOT}/build/fstools/bin/redoxfs"
DISK_IMAGE="$1"
MOUNT_POINT="/tmp/redoxfs-mount-$$"

if [[ -z "$DISK_IMAGE" ]]; then
    echo "Usage: $0 <disk-image>"
    exit 1
fi

if [[ ! -f "$DISK_IMAGE" ]]; then
    echo "Error: Disk image not found: $DISK_IMAGE"
    exit 1
fi

# Check if macFUSE is installed
if ! brew list --cask macfuse &>/dev/null; then
    echo "Installing macFUSE..."
    brew install --cask macfuse
    echo ""
    echo "macFUSE installed. You may need to:"
    echo "1. Allow the kernel extension in System Settings > Privacy & Security"
    echo "2. Restart your computer"
    echo "3. Run this script again"
    exit 1
fi

# Find the RedoxFS partition offset
echo "Analyzing disk image..."

# Write Python script to temp file
cat > /tmp/parse_gpt.py <<'PYTHON_EOF'
import struct
import sys

if len(sys.argv) < 2:
    print("Usage: parse_gpt.py <disk_image>", file=sys.stderr)
    sys.exit(1)

disk_image = sys.argv[1]

# Read GPT header (LBA 1, 512 bytes per sector)
with open(disk_image, 'rb') as f:
    # Skip MBR (LBA 0)
    f.seek(512)

    # Read GPT header
    gpt_header = f.read(512)

    # GPT signature should be "EFI PART"
    signature = gpt_header[0:8]
    if signature != b'EFI PART':
        print(f"Error: Invalid GPT signature: {signature}", file=sys.stderr)
        sys.exit(1)

    # Partition entry LBA and count
    part_entry_lba = struct.unpack('<Q', gpt_header[72:80])[0]
    num_entries = struct.unpack('<I', gpt_header[80:84])[0]
    entry_size = struct.unpack('<I', gpt_header[84:88])[0]

    # Read partition entries
    f.seek(part_entry_lba * 512)

    for i in range(num_entries):
        entry = f.read(entry_size)

        # Check if partition is used (non-zero type GUID)
        type_guid = entry[0:16]
        if type_guid == b'\x00' * 16:
            continue

        # Get partition name (UTF-16LE)
        part_name_bytes = entry[56:128]
        part_name = part_name_bytes.decode('utf-16le').rstrip('\x00')

        # Get start LBA
        start_lba = struct.unpack('<Q', entry[32:40])[0]
        end_lba = struct.unpack('<Q', entry[40:48])[0]

        print(f"Partition {i}: {part_name}")
        print(f"  Start LBA: {start_lba} (offset: {start_lba * 512} bytes)")
        print(f"  End LBA: {end_lba}")
        print(f"  Size: {(end_lba - start_lba + 1) * 512 // 1024 // 1024} MB")

        # Look for REDOXFS partition
        if 'REDOX' in part_name.upper():
            print(f"\nFound RedoxFS partition: {part_name}")
            print(f"START_OFFSET={start_lba * 512}")
            print(f"START_LBA={start_lba}")
            sys.exit(0)

print("Error: No RedoxFS partition found", file=sys.stderr)
sys.exit(1)
PYTHON_EOF

python3 /tmp/parse_gpt.py "$DISK_IMAGE" > /tmp/partition_info.txt 2>&1
PARSE_EXIT=$?

# Extract partition info
cat /tmp/partition_info.txt

if [[ $PARSE_EXIT -ne 0 ]]; then
    exit 1
fi

eval "$(grep "^START_" /tmp/partition_info.txt)"

echo ""
echo "Found partition at LBA $START_LBA (offset $START_OFFSET bytes)"
echo "Extracting RedoxFS partition..."
PARTITION_FILE="/tmp/redoxfs-partition-$$.img"
dd if="$DISK_IMAGE" of="$PARTITION_FILE" bs=512 skip="$START_LBA" status=progress

echo ""
echo "Mounting RedoxFS partition..."
mkdir -p "$MOUNT_POINT"

# Mount with redoxfs
"$REDOXFS" "$PARTITION_FILE" "$MOUNT_POINT" &
REDOXFS_PID=$!

# Wait for mount
sleep 2

if ! mountpoint -q "$MOUNT_POINT" 2>/dev/null; then
    echo "Error: Failed to mount RedoxFS"
    kill $REDOXFS_PID 2>/dev/null || true
    rm -f "$PARTITION_FILE"
    exit 1
fi

echo "Successfully mounted at $MOUNT_POINT"
echo ""
echo "Creating Ion configuration..."

# Create config directory
mkdir -p "$MOUNT_POINT/home/user/.config/ion"

# Create ionrc
cat > "$MOUNT_POINT/home/user/.config/ion/ionrc" <<'EOF'
export PROMPT = "redox> "
export LC_ALL = "C"
export LANG = "C"
export TERM = "vt100"
EOF

echo "Ion configuration injected!"
echo ""
echo "Unmounting..."

# Unmount
fusermount -u "$MOUNT_POINT" 2>/dev/null || umount "$MOUNT_POINT" 2>/dev/null || kill $REDOXFS_PID

sleep 1

echo "Writing partition back to disk..."
dd if="$PARTITION_FILE" of="$DISK_IMAGE" bs=512 seek=$START_LBA conv=notrunc status=progress

# Cleanup
rm -f "$PARTITION_FILE"
rm -rf "$MOUNT_POINT"

echo ""
echo "Done! Ion configuration has been injected into $DISK_IMAGE"
