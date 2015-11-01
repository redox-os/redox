#!/bin/bash
source environ.sh

BROKEN

SRC=ftp://ftp.freedesktop.org/pub/mesa/11.0.4/mesa-11.0.4.tar.xz
DIR=mesa-11.0.4

CONFIGURE_ARGS="--host=i386-elf-redox --prefix=$PREFIX"
autoconf_template $*
