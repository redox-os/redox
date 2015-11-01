#!/bin/bash
source environ.sh

BROKEN
DEPENDS libxml2

SRC=http://download.gna.org/warmux/warmux-11.04.1.tar.bz2
DIR=warmux-11.04

CONFIGURE_ARGS="--host=${HOST} --prefix=${PREFIX}"
configure_template $*
