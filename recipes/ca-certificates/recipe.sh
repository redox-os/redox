GIT="https://gitlab.redox-os.org/redox-os/ca-certificates.git"

function recipe_version {
    date "+%Y%m%d"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    rm -rf build
    mkdir build
    curl \
        -o certdata.txt \
        --time-cond certdata.txt \
        https://hg.mozilla.org/releases/mozilla-release/raw-file/default/security/nss/lib/ckfw/builtins/certdata.txt
    ./make-ca.sh -D "$PWD/build"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    rm -rf build
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -p "$1/ssl"
    cp -RL build/etc/ssl/certs "$1/ssl"
    skip=1
}
