#!/bin/bash
source environ.sh

BROKEN

GIT=https://github.com/mamedev/mame.git
DIR=mame

BUILD_ARGS="ARCHITECTURE=_x86"
make_template $*
