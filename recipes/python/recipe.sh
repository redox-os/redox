VERSION=3.7.4
TAR=https://www.python.org/ftp/python/$VERSION/Python-$VERSION.tar.xz
BUILD_DEPENDS=(openssl)

export CONFIG_SITE=config.site

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    export LDFLAGS="-static"
    cp ../config.site ./
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --build=${ARCH} \
        --prefix=/ \
        --disable-ipv6
    sed -i 's|#define HAVE_PTHREAD_KILL 1|/* #undef HAVE_PTHREAD_KILL */|g' pyconfig.h
    sed -i 's|#define HAVE_SCHED_SETSCHEDULER 1|/* #undef HAVE_SCHED_SETSCHEDULER */|g' pyconfig.h
    sed -i 's|#define HAVE_SYS_RESOURCE_H 1|/* #undef HAVE_SYS_RESOURCE_H */|g' pyconfig.h
    make -j"$(nproc)"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install -j"$(nproc)"
    "$STRIP" "$dest/bin/python3.7" "$dest/bin/python3.7m"
    skip=1
}
