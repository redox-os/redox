#!/bin/bash
source environ.sh

UNSTABLE

SRC=http://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.14.tar.gz
DIR=libiconv-1.14

CONFIGURE_ARGS="--host=${HOST} --prefix=${PREFIX} --disable-shared"
configure_template $*
