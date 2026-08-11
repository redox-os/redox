#!/usr/bin/env bash

# This script print all recipe configuration files with a determined text

bat --decorations=always $(rg "$1" -li --sort=path recipes)
