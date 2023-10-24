VERSION=8.2
# We use `https://ftp.nluug.nl` because `ftp.vim.org` doesn't support `https`
# see https://www.vim.org/mirrors.php
TAR=https://ftp.nluug.nl/pub/vim/unix/vim-$VERSION.tar.bz2

BUILD_DEPENDS=(ncurses)
DEPENDS="terminfo"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib -static"
    export CPPFLAGS="-I$sysroot/include"
    export vim_cv_toupper_broken=no
    export vim_cv_tgetent=zero
    export vim_cv_terminfo=yes
    export vim_cv_tty_group=world
    export vim_cv_getcwd_broken=no
    export vim_cv_stat_ignores_slash=yes
    export vim_cv_memmove_handles_overlap=yes
    ./configure --build=${BUILD} --host=${HOST} --prefix=/ --with-tlib=ncurses
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" ${MAKEFLAGS} install
    skip=1
}
