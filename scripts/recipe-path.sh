#!/usr/bin/env bash

# This script print the location of recipes

FIND_RECIPE="find cookbook/recipes -maxdepth 4 -name"

for recipe in $*
do
    ${FIND_RECIPE} "${recipe}"
done
