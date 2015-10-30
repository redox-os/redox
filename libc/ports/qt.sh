#!/bin/bash
source environ.sh

SRC=http://download.qt.io/official_releases/qt/5.5/5.5.1/single/qt-everywhere-opensource-src-5.5.1.tar.gz
DIR=qt-everywhere-opensource-src-5.5.1

CONFIGURE_ARGS="-xplatform redox-g++ -prefix $PREFIX"
configure_template $*
