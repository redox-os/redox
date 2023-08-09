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
    recipes="$(target/release/list_recipes)"
fi

for recipe in $recipes
do
    recipe_path=`target/release/find_recipe $recipe`
    echo recipe path is $recipe_path
    COOKBOOK_RECIPE="$recipe_path"
    TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
    COOKBOOK_BUILD="${TARGET_DIR}/build"
    COOKBOOK_STAGE="${TARGET_DIR}/stage"
    COOKBOOK_SOURCE="${COOKBOOK_RECIPE}/source"
    COOKBOOK_SYSROOT="${TARGET_DIR}/sysroot"

    if [ -e "${COOKBOOK_RECIPE}/recipe.toml" ]
    then
        target/release/cook "$recipe"

        if [ ! -f "${COOKBOOK_STAGE}.tar.gz" ]
        then
            echo -e "\033[01;38;5;155mrepo - legacy packaging $recipe\033[0m" >&2
            ./cook.sh "$recipe" tar $DEBUG
        else
            TIME_PKG="$($STAT -c "%Y" "${COOKBOOK_STAGE}.pkgar")"
            TIME_STAGE="$($STAT -c "%Y" "${COOKBOOK_STAGE}.tar.gz")"
            if [ "$TIME_PKG" -gt "$TIME_STAGE" ]
            then
                echo -e "\033[01;38;5;155mrepo - legacy repackaging $recipe\033[0m" >&2
                ./cook.sh "$recipe" untar tar $DEBUG
            fi
        fi

        # Match pkgar and tar time
        touch -c -r "${COOKBOOK_STAGE}.tar.gz" "${COOKBOOK_STAGE}.pkgar"

        continue
    fi

    if [ ! -d "${COOKBOOK_SOURCE}" ]
    then
        echo -e "\033[01;38;5;155mrepo - fetching $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -d "${COOKBOOK_BUILD}" ]
    then
        echo -e "\033[01;38;5;155mrepo - preparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" prepare
    elif [ ! -d "${COOKBOOK_SYSROOT}" ]
    then
        echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" unprepare prepare
    else
        TIME_SOURCE="$($FIND "${COOKBOOK_SOURCE}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_BUILD="$($FIND "${COOKBOOK_BUILD}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        if [ "$TIME_SOURCE" -gt "$TIME_BUILD" ]
        then
            echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
            ./cook.sh "$recipe" unprepare prepare
        fi
    fi

    if [ ! -f "${COOKBOOK_STAGE}.tar.gz" ]
    then
        echo -e "\033[01;38;5;155mrepo - building $recipe\033[0m" >&2
        ./cook.sh "$recipe" build stage tar $DEBUG
    else
        TIME_BUILD="$($FIND "${COOKBOOK_BUILD}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_STAGE="$($STAT -c "%Y" "${COOKBOOK_STAGE}.tar.gz")"
        TIME_RECIPE="$($FIND "${COOKBOOK_RECIPE}"/{recipe.sh,*.patch} -printf '%Ts\n' | sort -nr | head -n 1)"
        if [ "$TIME_BUILD" -gt "$TIME_STAGE" -o "$TIME_RECIPE" -gt "$TIME_STAGE" ]
        then
            echo -e "\033[01;38;5;155mrepo - rebuilding $recipe\033[0m" >&2
            ./cook.sh "$recipe" untar unstage build stage tar $DEBUG
        else
            echo -e "\033[01;38;5;155mrepo - $recipe up to date\033[0m" >&2
        fi
    fi

    if [ ! -f "${COOKBOOK_STAGE}.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - packaging $recipe\033[0m" >&2
        ./cook.sh "$recipe" pkg $DEBUG
    else
        TIME_STAGE="$($STAT -c "%Y" "${COOKBOOK_STAGE}.tar.gz")"
        TIME_PKG="$($STAT -c "%Y" "${COOKBOOK_STAGE}.pkgar")"
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
    recipe_path=`target/release/find_recipe $recipe`
    COOKBOOK_RECIPE="$recipe_path"
    TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
    COOKBOOK_STAGE="${TARGET_DIR}/stage"

    if [ "${COOKBOOK_STAGE}.tar.gz" -nt "$REPO/$recipe.tar.gz" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        ./cook.sh $recipe publish
    fi

    if [ "${COOKBOOK_STAGE}.pkgar" -nt "$REPO/$recipe.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        cp -v "${COOKBOOK_STAGE}.pkgar" "$REPO/$recipe.pkgar"
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
