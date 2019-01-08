BUILD_DEPENDS=(liborbital cairo pixman zlib libpng freetype)

function recipe_version {
    printf "1.0.0"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_prepare {
    rm -rf source
    mkdir source
    cp cairodemo.c source
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include"
    set -x
    "${CXX}" -I "$sysroot/include" -L "$sysroot/lib" cairodemo.c -o cairodemo -lorbital -lcairo -lpixman-1 -lfreetype -lpng -lz -lm
    set +x
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
    mkdir -pv "$dest/bin"
    cp -v "cairodemo" "$dest/bin/cairodemo"
    skip=1
}
