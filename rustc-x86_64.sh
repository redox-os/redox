#!/bin/bash
RUST_BACKTRACE=1 rustc -L build/x86_64-unknown-redox/debug $*
