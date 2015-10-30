#!/bin/bash
source environ.sh

SRC=http://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.6.tar.xz
DIR=gettext-0.19.6

CONFIGURE_ARGS="--host=i386-elf-redox --prefix=$PREFIX"
configure_template $*
