#!/bin/bash
source environ.sh

STABLE

SRC=http://www.lua.org/ftp/lua-5.3.1.tar.gz
DIR=lua-5.3.1

BUILD_ARGS="generic"
INSTALL_ARGS="INSTALL_TOP=${PREFIX}"
make_template $*
