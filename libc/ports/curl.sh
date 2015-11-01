#!/bin/bash
source environ.sh

BROKEN

SRC=http://curl.haxx.se/download/curl-7.45.0.tar.gz
DIR=curl-7.45.0

CONFIGURE_ARGS="--host=i386-elf-redox --prefix=$PREFIX"
configure_template $*
