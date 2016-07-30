#!/bin/bash
RUST_BACKTRACE=1 rustc --cfg redox -L build/i386-unknown-redox/debug $*
