#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

recipes=()
for arg in "${@:1}"
do
    if [ "$arg" == "--debug" ]
    then
        DEBUG=--debug
    else
        recipes[${#recipes[@]}]="$arg"
    fi
done

if [ ${#recipes[@]} = 0 ]
then
    recipes="$(ls -1 recipes)"
fi

for recipe in $recipes
do
    if [ ! -d "recipes/$recipe/source/" ]
    then
        echo -e "\033[01;38;5;215mrepo - fetching $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -d "recipes/$recipe/build/" ]
    then
        echo -e "\033[01;38;5;155mrepo - preparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" prepare
    else
        TIME_SOURCE="$($FIND recipes/$recipe/source/ -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_BUILD="$($FIND recipes/$recipe/build/ -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        if [ "$TIME_SOURCE" -gt "$TIME_BUILD" ]
        then
            echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
            ./cook.sh "$recipe" unprepare prepare
        fi
    fi

    if [ ! -f "recipes/$recipe/stage.tar.gz" ]
    then
        echo -e "\033[01;38;5;155mrepo - building $recipe\033[0m" >&2
        ./cook.sh "$recipe" build stage tar $DEBUG
    else
        TIME_BUILD="$($FIND recipes/$recipe/build/ -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_STAGE="$($STAT -c "%Y" recipes/$recipe/stage.tar.gz)"
        TIME_RECIPE="$($FIND recipes/$recipe/{recipe.sh,*.patch} -printf '%Ts\n' | sort -nr | head -n 1)"
        if [ "$TIME_BUILD" -gt "$TIME_STAGE" -o "$TIME_RECIPE" -gt "$TIME_STAGE" ]
        then
            echo -e "\033[01;38;5;155mrepo - rebuilding $recipe\033[0m" >&2
            ./cook.sh "$recipe" untar unstage build stage tar $DEBUG
        else
            echo -e "\033[01;38;5;155mrepo - $recipe up to date\033[0m" >&2
        fi
    fi
done

for recipe in $recipes
do
    if [ "recipes/$recipe/stage.tar.gz" -nt "$REPO/$recipe.tar.gz" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        ./cook.sh $recipe publish
    fi
done

echo -e "\033[01;38;5;155mrepo - generating repo.toml\033[0m" >&2

echo "[packages]" > "$REPO/repo.toml"
for toml in "$REPO/"*".toml"
do
    package="$(basename "$toml" .toml)"
    if [ "$package" != "repo" ]
    then
        version="$(grep version "$toml" | cut -d '=' -f2-)"
        echo "$package =$version" >> "$REPO/repo.toml"
    fi
done
