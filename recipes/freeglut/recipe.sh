TAR=https://cytranet.dl.sourceforge.net/project/freeglut/freeglut/3.0.0/freeglut-3.0.0.tar.gz

BUILD_DEPENDS=(mesa mesa_glu)

function recipe_version {
    echo "3.0.0"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    cmake \
      -D CMAKE_TOOLCHAIN_FILE=../redox_cross_toolchain.cmake \
      -D CMAKE_INSTALL_PREFIX=/ \
      -D FREEGLUT_GLES=0 \
      .
    #./configure --host="${HOST}" --prefix=/ --enable-osmesa
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    skip=1
}
