#!/bin/bash
source environ.sh

STABLE

SRC=http://downloads.sourceforge.net/project/expat/expat/2.1.0/expat-2.1.0.tar.gz
DIR=expat-2.1.0

CONFIGURE_ARGS="--host=${HOST} --prefix=${PREFIX}"
configure_template $*
