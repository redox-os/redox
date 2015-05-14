#!/bin/bash
sudo apt-get install build-essential libc6-dev-i386 nasm qemu-system-x86 qemu-kvm
git clone https://github.com/rust-lang/rust.git
cd rust
./configure --target=i686-unknown-linux-gnu
make
sudo make install
