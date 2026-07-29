#!/usr/bin/env bash

# This script show all unused packages in given category, or "wip/libs" by default.

# This script must be inside podman `make env`

CATEGORY=${1:-wip/libs}

target/release/repo cook-list --all --display=csv |  grep ",$" |  grep ",recipes/$CATEGORY" | cut -d ',' -f2
