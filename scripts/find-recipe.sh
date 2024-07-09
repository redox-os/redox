# This script show all files installed by a recipe

#!/usr/bin/env bash

# Ensure arch and config are set as desired, we use these to find the build dir
export ARCH=$(uname -m)
export CONFIG_NAME=desktop

# Make sure to unmount the image first
make unmount &>/dev/null || true

# Mount the image
make mount >/dev/null

# Find all files
find "build/${ARCH}/${CONFIG_NAME}/" -type f | cut -d / -f5- |\
sort |\
uniq |\
while read path
do
    # Skip empty paths
    if [ -z "${path}" ]
    then
        continue
    fi

    # Find all packages providing this file
    pkgs="$(
        find cookbook/recipes/*"/target/${ARCH}-unknown-redox/stage/${path}" 2>/dev/null |
        cut -d/ -f3 |
        tr '\n' ' ' |
        sort |
        uniq
    )"
    if [ -n "${pkgs}" ]
    then
        echo "$path: ${pkgs}"
    else
        echo "$path: no packages, see config/${ARCH}/${CONFIG_NAME}.toml"
    fi
done

# Make sure to unmount the image
make unmount &>/dev/null || true
