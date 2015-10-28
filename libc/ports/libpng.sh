#!/bin/bash
source environ.sh

SRC=http://download.sourceforge.net/libpng/libpng-1.2.52.tar.gz
DIR=libpng-1.2.52

CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
