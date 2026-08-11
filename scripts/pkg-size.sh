#!/usr/bin/env bash

# This script show the size of some recipe package size ("stage.pkgar")
# This script must be inside podman `make env`

RECIPES=$(target/release/repo cook-list ${*:---all-compiled} --display=csv |  grep ",Built" | sort -t ',' -k 3 -V | cut -d ',' -f2)
TARGET=$(target/release/cookbook_redoxer --target)
for recipe in $RECIPES
do
    ls -hs "$recipe/target/$TARGET/stage.pkgar"
done
