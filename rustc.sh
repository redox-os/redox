#!/bin/bash
RUST_BACKTRACE=1 rustc -L build/userspace $*
