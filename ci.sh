#!/bin/bash

set -e

./update-packages.sh
./all.sh publish
