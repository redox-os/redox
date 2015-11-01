#!/bin/bash
source environ.sh

BROKEN

SRC=ftp://xmlsoft.org/libxml2/libxml2-2.9.2.tar.gz
DIR=libxml2-2.9.2

CONFIGURE_ARGS="--host=i386-elf-redox --prefix=$PREFIX"
configure_template $*
