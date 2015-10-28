#!/bin/bash
set -e

mkdir -p build

PREFIX="${PWD}/build/prefix"
mkdir -p "${PREFIX}"
mkdir -p "${PREFIX}/bin"
export PATH="${PREFIX}/bin:$PATH"

SYSROOT="${PWD}/build/sysroot"
mkdir -p "${SYSROOT}"
