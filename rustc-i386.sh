#!/bin/bash
RUST_BACKTRACE=1 rustc -L build/i386-unknown-redox/debug $*
