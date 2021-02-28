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
    export LDFLAGS="-L$sysroot/lib --static"
    wget -O build-aux/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static \
        ac_cv_have_decl_program_invocation_name=no \
        gt_cv_locale_fr=false \
        gt_cv_locale_fr_utf8=false \
        gt_cv_locale_ja=false \
        gt_cv_locale_tr_utf8=false \
        gt_cv_locale_zh_CN=false
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
