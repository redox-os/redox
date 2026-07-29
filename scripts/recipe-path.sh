#!/usr/bin/env bash

# Return list of paths from recipes

IFS=","
make "find.$*"
