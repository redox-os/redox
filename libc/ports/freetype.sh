#!/bin/bash
source environ.sh

SRC=http://download.savannah.gnu.org/releases/freetype/freetype-2.6.1.tar.gz
DIR=freetype-2.6.1

CONFIGURE_ARGS="--host=${HOST} --disable-shared --with-harfbuzz=no"
configure_template $*
