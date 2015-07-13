#!/bin/bash
git clone https://github.com/rust-lang/rust.git
cd rust
./configure --target=i686-unknown-linux-gnu
make
sudo make install
