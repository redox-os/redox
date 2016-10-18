#!/bin/bash
RUST_BACKTRACE=1 rustdoc -L build/kernel --cfg redox $*
