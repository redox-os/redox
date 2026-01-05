#!/bin/bash
# Native build script for macOS
# Sets up GNU coreutils and cargo bin in PATH, plus library paths for GMP/MPFR

export PATH="/opt/homebrew/opt/coreutils/libexec/gnubin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CPPFLAGS="-I/opt/homebrew/opt/gmp/include -I/opt/homebrew/opt/mpfr/include"
export LDFLAGS="-L/opt/homebrew/opt/gmp/lib -L/opt/homebrew/opt/mpfr/lib"

# Run make with all arguments passed to this script
make "$@"
