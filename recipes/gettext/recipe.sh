VERSION=0.19.8.1
TAR=http://ftp.gnu.org/pub/gnu/gettext/gettext-${VERSION}.tar.xz
BUILD_DEPENDS=(libiconv)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    wget -O build-aux/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static \
        gt_cv_locale_fr=false \
        gt_cv_locale_fr_utf8=false \
        gt_cv_locale_ja=false \
        gt_cv_locale_tr_utf8=false \
        gt_cv_locale_zh_CN=false
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
    make DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
