#!/usr/bin/env bash
set -e

# Configuration
export TARGET=x86_64-unknown-redox

# Automatic variables
ROOT="$(cd `dirname "$0"` && pwd)"
REPO="$ROOT/repo/$TARGET"
export CC="x86_64-elf-redox-gcc"
export XARGO_HOME="$ROOT/xargo-home"
