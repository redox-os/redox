#!/bin/bash
source environ.sh

DIR=zlib-1.2.8
CONFIGURE_ARGS="--static"
configure_template $*
