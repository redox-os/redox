#!/bin/bash
source environ.sh

cd lua-5.3.1
make -j `nproc`
