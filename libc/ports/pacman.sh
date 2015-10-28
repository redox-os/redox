#!/bin/bash
source environ.sh

DIR=pacman-master/src
CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
