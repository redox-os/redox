#!/bin/bash -e

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
        echo -e "\033[01;38;5;215mrepo - fetching and updating $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -f "recipes/$recipe/stage.tar" ]
    then
        echo -e "\033[01;38;5;215mrepo - building $recipe\033[0m" >&2
        ./cook.sh $recipe update build stage tar
    else
        TIME_SOURCE="$(find recipes/$recipe/source -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_STAGE="$(stat -c "%Y" recipes/$recipe/stage.tar)"
        if [ "$TIME_SOURCE" -ge "$TIME_STAGE" ]
        then
            echo -e "\033[01;38;5;215mrepo - rebuilding $recipe\033[0m" >&2
            ./cook.sh "$recipe" untar unstage update build stage tar
        else
            echo -e "\033[01;38;5;215mrepo - $recipe up to date\033[0m" >&2
        fi
    fi
done

for recipe in $recipes
do
    if [ "recipes/$recipe/stage.tar" -nt "$REPO/$recipe.tar" ]
    then
        echo -e "\033[01;38;5;215mrepo - publishing $recipe\033[0m" >&2
        ./cook.sh $recipe publish
    fi
done

echo -e "\033[01;38;5;215mrepo - generating repo.toml\033[0m" >&2

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
