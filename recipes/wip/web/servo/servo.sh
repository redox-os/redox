#!/usr/bin/env bash

# bug within egui/llvm causing broken fonts
export GALLIUM_DRIVER=softpipe

exec /usr/lib/servo/bin/servoshell
