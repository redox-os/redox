function recipe_fetch {
    if [ ! -d source ]
    then
        mkdir source
    fi
    pushd source
        curl -o make-ca.sh --time-cond make-ca.sh http://anduin.linuxfromscratch.org/BLFS/other/make-ca.sh-20170514
        curl -o certdata.txt --time-cond certdata.txt http://anduin.linuxfromscratch.org/BLFS/other/certdata.txt
    popd
    skip=1
}

function recipe_update {
    skip=1
}

function recipe_build {
    rm -rf build
    mkdir build
    chmod +x ./make-ca.sh
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
    cp -rL build/etc/ssl/certs "$1/ssl"
    skip=1
}
