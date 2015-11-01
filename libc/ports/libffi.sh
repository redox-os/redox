#!/bin/bash
source environ.sh

STABLE

SRC=ftp://sourceware.org/pub/libffi/libffi-3.2.1.tar.gz
DIR=libffi-3.2.1

CONFIGURE_ARGS="--host=i386-elf-redox --prefix=$PREFIX"
configure_template $*
