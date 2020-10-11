#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--debug" ]
    then
        DEBUG=--debug
    else
        recipes+=" $arg"
    fi
done

if [ "$recipes" == "" ]
then
    recipes="$(ls -1 recipes)"
fi

for recipe in $recipes
do
    if [ -e "recipes/$recipe/recipe.toml" ]
    then
        target/release/cook "$recipe"

        if [ ! -f "recipes/$recipe/stage.tar.gz" ]
        then
            echo -e "\033[01;38;5;155mrepo - legacy packaging $recipe\033[0m" >&2
            ./cook.sh "$recipe" tar $DEBUG
        else
            TIME_PKG="$($STAT -c "%Y" recipes/$recipe/stage.pkgar)"
            TIME_STAGE="$($STAT -c "%Y" recipes/$recipe/stage.tar.gz)"
            if [ "$TIME_PKG" -gt "$TIME_STAGE" ]
            then
                echo -e "\033[01;38;5;155mrepo - legacy repackaging $recipe\033[0m" >&2
                ./cook.sh "$recipe" untar tar $DEBUG
            fi
        fi

        # Match pkgar and tar time
        touch --no-create --reference="recipes/$recipe/stage.tar.gz" "recipes/$recipe/stage.pkgar"

        continue
    fi

    if [ ! -d "recipes/$recipe/source/" ]
    then
        echo -e "\033[01;38;5;155mrepo - fetching $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -d "recipes/$recipe/build/" ]
    then
        echo -e "\033[01;38;5;155mrepo - preparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" prepare
    elif [ ! -d "recipes/$recipe/sysroot/" ]
    then
        echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" unprepare prepare
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

    if [ ! -f "recipes/$recipe/stage.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - packaging $recipe\033[0m" >&2
        ./cook.sh "$recipe" pkg $DEBUG
    else
        TIME_STAGE="$($STAT -c "%Y" recipes/$recipe/stage.tar.gz)"
        TIME_PKG="$($STAT -c "%Y" recipes/$recipe/stage.pkgar)"
        if [ "$TIME_STAGE" -gt "$TIME_PKG" ]
        then
            echo -e "\033[01;38;5;155mrepo - repackaging $recipe\033[0m" >&2
            ./cook.sh "$recipe" unpkg pkg $DEBUG
        fi
    fi
done

mkdir -p "$REPO"

for recipe in $recipes
do
    if [ "recipes/$recipe/stage.tar.gz" -nt "$REPO/$recipe.tar.gz" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        ./cook.sh $recipe publish
    fi

    if [ "recipes/$recipe/stage.pkgar" -nt "$REPO/$recipe.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        cp -v "recipes/$recipe/stage.pkgar" "$REPO/$recipe.pkgar"
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
