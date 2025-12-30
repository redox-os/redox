#!/bin/bash
# Check Redox build status

echo "=== Redox Build Status ==="
echo ""

# Check if build is running
if ps aux | grep -E "build-minimal|make.*ARCH=aarch64" | grep -v grep > /dev/null; then
    echo "✅ Build is RUNNING"
    echo ""
    echo "Recent build output:"
    echo "-------------------"
    tail -20 /tmp/claude/-opt-other-redox/tasks/ba78120.output 2>/dev/null || echo "No output file found"
else
    echo "❌ Build is NOT running"
fi

echo ""
echo "=== Build Artifacts ==="
echo ""

# Check for built images
if [ -f "build/aarch64/minimal/harddrive.img" ]; then
    echo "✅ Minimal image built: $(ls -lh build/aarch64/minimal/harddrive.img | awk '{print $5}')"
elif [ -f "build/aarch64/minimal/redox-live.iso" ]; then
    echo "✅ Minimal ISO built: $(ls -lh build/aarch64/minimal/redox-live.iso | awk '{print $5}')"
else
    echo "⏳ No image built yet"
fi

if [ -f "build/aarch64/desktop/harddrive.img" ]; then
    echo "✅ Desktop image built: $(ls -lh build/aarch64/desktop/harddrive.img | awk '{print $5}')"
fi

echo ""
echo "To monitor build progress:"
echo "  tail -f /tmp/claude/-opt-other-redox/tasks/ba78120.output"
