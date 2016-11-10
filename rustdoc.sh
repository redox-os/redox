#!/bin/bash
RUST_BACKTRACE=1 rustdoc -L build/userspace $*
