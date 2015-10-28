#!/bin/bash
source environ.sh

DIR=libpng-1.2.52
CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
