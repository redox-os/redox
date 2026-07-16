#!/usr/bin/env bash

# This script print the recipe configuration

cat $(target/release/repo find "$1")/recipe.*
