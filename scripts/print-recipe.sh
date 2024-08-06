#!/usr/bin/env bash

# This script print the recipe configuration

cat "$(find cookbook/recipes -name $1)"/recipe.*
