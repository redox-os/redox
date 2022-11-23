#!/usr/bin/env bash

set -e

LAST_RELEASE_TAG="$(git describe --tags --abbrev=0)"
LAST_RELEASE_TIMESTAMP="$(git log --format="%ct" -1 "${LAST_RELEASE_TAG}")"
echo "Last release: ${LAST_RELEASE_TAG} at ${LAST_RELEASE_TIMESTAMP}"

REPOS=(
    .
    cookbook
)

for package in $(installer/target/release/redox_installer --list-packages -c config/x86_64/desktop.toml)
do
    REPOS+=("cookbook/recipes/${package}/source")
done

# TODO: resolve dependencies instead of manually adding these initfs packages
for package in init logd nulld ramfs randd zerod
do
    REPOS+=("cookbook/recipes/${package}/source")
done

for repo in "${REPOS[@]}"
do
    remote="$(git -C "${repo}" remote get-url origin)"
    website="${remote%.*}"
    name="$(basename "${website}")"
    before="$(git -C "${repo}" log --until="${LAST_RELEASE_TIMESTAMP}" --format="%h" -1)"
    after="$(git -C "${repo}" log --since="${LAST_RELEASE_TIMESTAMP}" --format="%h" -1)"
    echo -en "\x1B[1m${name}:\x1B[0m "
    if [ -z "${before}" ]
    then
        echo "New repository at ${website}"
    elif [ -z "${after}" ]
    then
        echo "No changes"
    else
        echo "${website}/-/compare/${before}...${after}"
    fi
done
