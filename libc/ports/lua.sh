#!/bin/bash
source environ.sh

DIR=lua-5.3.1
INSTALL_ARGS="INSTALL_TOP=${PREFIX}"
make_template $*
