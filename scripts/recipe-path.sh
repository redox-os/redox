#!/usr/bin/env bash

# This script print the location of recipes

cd cookbook

for recipe in $*
do
    target/release/find_recipe "$recipe"
done
