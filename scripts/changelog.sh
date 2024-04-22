#!/usr/bin/env bash

set -e

LAST_RELEASE_TAG="$(git describe --tags --abbrev=0)"
LAST_RELEASE_TIMESTAMP="$(git log --format="%ct" -1 "${LAST_RELEASE_TAG}")"
echo "Last release: ${LAST_RELEASE_TAG} at ${LAST_RELEASE_TIMESTAMP}"

REPOS=(
    redox=.
    cookbook=cookbook
    rust=rust
)

if [ "$1" = "--summary" ]
then
    summary=true
elif [ "$1" = "--mdlinks" ]
then
    mdlinks=true
fi

for package in $(installer/target/release/redox_installer --list-packages -c config/$(uname -m)/desktop.toml)
do
    package_source="$(cd cookbook; target/release/find_recipe ${package})"
    REPOS+=("${package}=cookbook/${package_source}/source")
done

# TODO: resolve dependencies instead of manually adding these initfs packages
for package in init logd ramfs randd zerod
do
    package_source="$(cd cookbook; target/release/find_recipe ${package})"
    REPOS+=("${package}=cookbook/${package_source}/source")
done

for name_repo in "${REPOS[@]}"
do
    name="$(echo "${name_repo}" | cut -d "=" -f 1)"
    repo="$(echo "${name_repo}" | cut -d "=" -f 2-)"
    if [ "${summary}" = true ]
    then
        echo
        echo "### ${name}"
        echo
    elif [ "${mdlinks}" = true ]
    then
        echo -n "- [${name}]"
    else
        echo -en "\x1B[1m${name}:\x1B[0m "
    fi

    if [ -e "${repo}/.git" ]
    then
        remote="$(git -C "${repo}" remote get-url origin)"
        website="${remote%.*}"
        before="$(git -C "${repo}" log --until="${LAST_RELEASE_TIMESTAMP}" --format="%h" -1)"
        after="$(git -C "${repo}" log --since="${LAST_RELEASE_TIMESTAMP}" --format="%h" -1)"
        if [ -z "${before}" ]
        then
            echo "New repository at ${website}"
        elif [ -z "${after}" ]
        then
            echo "No changes"
        else
            if [ "${summary}" = true ]
            then
                git -C "${repo}" log ${before}...${after} --oneline
            elif [ "${mdlinks}" = true ]
            then
                echo "(${website}/-/compare/${before}...${after})"
            else
                echo "${website}/-/compare/${before}...${after}"
            fi
        fi
    else
        echo "Not a git repository"
    fi
done
