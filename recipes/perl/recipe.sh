VERSION=5.24.2
TAR=http://www.cpan.org/src/5.0/perl-$VERSION.tar.gz

unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    curl -L -O --time-cond perl-cross-1.1.6.tar.gz https://github.com/arsv/perl-cross/releases/download/1.1.6/perl-cross-1.1.6.tar.gz
    tar --strip-components=1 -xvf perl-cross-1.1.6.tar.gz
    wget -O cnf/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    sysroot="$($HOST-gcc -print-sysroot)"
    ./configure --target=${HOST} --prefix='/' --sysroot="$sysroot" --disable-mod=Sys-Syslog,POSIX,Time-HiRes --with-libs='m'
    sed -i "s/^#define Netdb_name_t.*/#define Netdb_name_t const char*/" config.h # XXX
    sed -i 's/#define Strerror(e).*$/#define Strerror(e) strerror(e)/' config.h #
    echo "#define HAS_VPRINTF" >> config.h
    make LIBS=-lm
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
    rm -rf "$1/man"
    skip=1
}
