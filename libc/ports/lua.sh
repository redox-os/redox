#!/bin/bash
source environ.sh

SRC=http://www.lua.org/ftp/lua-5.3.1.tar.gz
DIR=lua-5.3.1

INSTALL_ARGS="INSTALL_TOP=${PREFIX}"
make_template $*
