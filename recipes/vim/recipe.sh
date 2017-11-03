VERSION=8.0.586
TAR=http://ftp.vim.org/vim/unix/vim-$VERSION.tar.bz2
BUILD_DEPENDS=(ncurses)

export AR="${HOST}-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-ld"
export NM="${HOST}-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export RANLIB="${HOST}-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include"
    export vim_cv_toupper_broken=set
    export vim_cv_terminfo=no
    export vim_cv_tty_group=world
    export vim_cv_getcwd_broken=yes
    export vim_cv_stat_ignores_slash=no
    export vim_cv_memmove_handles_overlap=yes
    ./configure --host=${HOST} --prefix=/ --with-tlib=ncurses
    make 
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
    make DESTDIR="$dest" ${MAKEFLAGS} install
    skip=1
}
