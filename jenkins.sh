#!/bin/bash

set -e

./all.sh distclean
./all.sh dist
./all.sh publish
