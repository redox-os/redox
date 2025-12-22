#!/usr/bin/env bash

# This script print the recipe configuration

cat $(target/release/find_recipe "$1")/recipe.*
