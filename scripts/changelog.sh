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

for package in $(installer/target/release/redox_installer --list-packages -c config/$(uname -m)/desktop.toml)
do
    REPOS+=("${package}=cookbook/recipes/${package}/source")
done

# TODO: resolve dependencies instead of manually adding these initfs packages
for package in init logd nulld ramfs randd zerod
do
    REPOS+=("${package}=cookbook/recipes/${package}/source")
done

for name_repo in "${REPOS[@]}"
do
    name="$(echo "${name_repo}" | cut -d "=" -f 1)"
    repo="$(echo "${name_repo}" | cut -d "=" -f 2-)"
    echo -en "\x1B[1m${name}:\x1B[0m "
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
            echo "${website}/-/compare/${before}...${after}"
        fi
    else
        echo "Not a git repository"
    fi
done
