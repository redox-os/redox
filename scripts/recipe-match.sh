#!/usr/bin/env bash

# This script print the recipe configuration files with determined text

bat --decorations=always $(rg "$1" -li --sort=path cookbook/recipes)
