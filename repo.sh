#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(ls -1 recipes)"
else
    recipes="$@"
fi

for recipe in $recipes
do
    if [ ! -d "recipes/$recipe/source" ]
    then
        echo -e "\033[01;38;5;215mrepo - fetching $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -d "recipes/$recipe/build" ]
    then
        echo -e "\033[01;38;5;155mrepo - preparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" prepare
    else
        TIME_SOURCE="$(find recipes/$recipe/source -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_BUILD="$(find recipes/$recipe/build -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        if [ "$TIME_SOURCE" -gt "$TIME_BUILD" ]
        then
            echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
            ./cook.sh "$recipe" unprepare prepare
        fi
    fi

    if [ ! -f "recipes/$recipe/stage.tar" ]
    then
        echo -e "\033[01;38;5;155mrepo - building $recipe\033[0m" >&2
        ./cook.sh "$recipe" update build stage tar
    else
        TIME_BUILD="$(find recipes/$recipe/build -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_STAGE="$(stat -c "%Y" recipes/$recipe/stage.tar)"
        TIME_RECIPE="$(find $(git ls-tree -r --name-only HEAD recipes/$recipe) -printf '%Ts\n' | sort -nr | head -n 1)"
	if [ "$TIME_BUILD" -gt "$TIME_STAGE" -o "$TIME_RECIPE" -gt "$TIME_STAGE" ]
        then
            echo -e "\033[01;38;5;155mrepo - rebuilding $recipe\033[0m" >&2
            ./cook.sh "$recipe" untar unstage update build stage tar
        else
            echo -e "\033[01;38;5;155mrepo - $recipe up to date\033[0m" >&2
        fi
    fi
done

for recipe in $recipes
do
    if [ "recipes/$recipe/stage.tar" -nt "$REPO/$recipe.tar" ]
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
