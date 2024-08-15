#!/usr/bin/env bash

# This script print the recipe configuration

cd cookbook

cat $(target/release/find_recipe "$1")/recipe.*
