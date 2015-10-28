#!/bin/bash
source environ.sh

SRC=http://zlib.net/zlib-1.2.8.tar.gz
DIR=zlib-1.2.8

CONFIGURE_ARGS="--static"
configure_template $*
