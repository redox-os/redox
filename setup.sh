#!/bin/bash

set -e

./cook.sh libstd unfetch
./cook.sh libstd fetch
./cook.sh libstd build
cp recipes/libstd/build/target/x86_64-unknown-redox/debug/deps/*.rlib ~/.xargo/lib/rustlib/x86_64-unknown-redox/lib/

echo "cook.sh is ready to use"
