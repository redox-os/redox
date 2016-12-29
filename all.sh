#!/bin/bash

set -e

for recipe in `ls -1 recipes`
do
    ./cook.sh $recipe $*
done
