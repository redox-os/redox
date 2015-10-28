#!/bin/bash
source environ.sh

DIR=freetype-2.6.1
CONFIGURE_ARGS="--host=${HOST} --disable-shared --with-harfbuzz=no"
configure_template $*
