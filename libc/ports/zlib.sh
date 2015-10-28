#!/bin/bash
source environ.sh

DIR=zlib-1.2.8
export CC=i386-elf-redox-gcc
CONFIGURE_ARGS="--static"
configure_template $*
