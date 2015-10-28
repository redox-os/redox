#!/bin/bash
source environ.sh

SRC=ftp://ftp.simplesystems.org/pub/libpng/png/src/libpng12/libpng-1.2.53.tar.gz
DIR=libpng-1.2.53

CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
