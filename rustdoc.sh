#!/usr/bin/env bash
RUST_BACKTRACE=1 rustdoc -L build/userspace $*
