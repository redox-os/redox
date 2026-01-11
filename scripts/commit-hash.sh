#!/usr/bin/env bash

# This script shows the current Git commit hash of system recipes at recipes/core

set -e

# Check if recipes/core directory exists
if [ ! -d "recipes/core" ]
then
    echo "Error: recipes/core directory not found"
    exit 1
fi

# Iterate through all system recipes in recipes/core
for recipe_dir in recipes/core/*/
do
    recipe_name=$(basename "$recipe_dir")
    source_dir="$recipe_dir/source"
    
    # Check if source directory exists and is a git repository
    if [ -d "$source_dir" ] && [ -d "$source_dir/.git" ]
    then
        # Get the commit hash
        commit_hash=$(cd "$source_dir" && git rev-parse HEAD)
        echo "$recipe_name: $commit_hash"
    fi
done
