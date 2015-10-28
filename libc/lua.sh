#!/bin/bash
export PATH=${PWD}/build/prefix/bin:${PATH}
cd lua-5.3.1
make -j `nproc`
