#!/bin/bash
source environ.sh

BROKEN

SRC=http://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-3.3.tar.gz
DIR=netsurf-all-3.3

BUILD_ARGS="HOST=${HOST} PREFIX=${PREFIX} TARGET=framebuffer"
make_template $*
