#!/bin/bash

set -e

echo "Defaulting to rust nightly"
rustup override set nightly
echo "Downloading rust source"
rustup component add rust-src
if [ -z "$(which xargo)" ]
then
    echo "Installing xargo"
    cargo install -f xargo
fi

echo "cook.sh is ready to use"
