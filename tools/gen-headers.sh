#!/bin/bash
# Generate relibc headers using cbindgen
# Usage: ./gen-headers.sh [arch]

set -e

ARCH="${1:-x86_64}"
RELIBC_DIR="/opt/other/redox/recipes/core/relibc/source"
TARGET_HEADERS="$RELIBC_DIR/target/${ARCH}-unknown-redox/include"

echo "Generating headers for $ARCH..."
echo "Output: $TARGET_HEADERS"

rm -rf "$TARGET_HEADERS"
mkdir -p "$TARGET_HEADERS"

# Copy static headers
cp -r "$RELIBC_DIR/include"/* "$TARGET_HEADERS/"
cp "$RELIBC_DIR/openlibm/include"/*.h "$TARGET_HEADERS/" 2>/dev/null || true
cp "$RELIBC_DIR/openlibm/src"/*.h "$TARGET_HEADERS/" 2>/dev/null || true

# Generate cbindgen headers
cd "$RELIBC_DIR"
for header_dir in src/header/*/; do
    name=$(basename "$header_dir")
    [[ "$name" == _* ]] && continue

    if [[ -f "$header_dir/cbindgen.toml" ]]; then
        out=$(echo "$name" | sed 's/_/\//g')
        out_path="$TARGET_HEADERS/${out}.h"
        mkdir -p "$(dirname "$out_path")"

        # Combine configs
        cat "$header_dir/cbindgen.toml" cbindgen.globdefs.toml > /tmp/cbindgen-combined.toml 2>/dev/null || \
            cat "$header_dir/cbindgen.toml" > /tmp/cbindgen-combined.toml

        if cbindgen "$header_dir/mod.rs" --config /tmp/cbindgen-combined.toml --output "$out_path" 2>/dev/null; then
            echo "  Generated $out.h"
        fi
    fi
done

echo "Headers generated: $TARGET_HEADERS"
