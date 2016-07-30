#!/bin/bash
RUST_BACKTRACE=1 rustc --cfg redox -L build/x86_64-unknown-redox/debug $*
