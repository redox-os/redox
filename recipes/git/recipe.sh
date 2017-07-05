VERSION=2.13.1
TAR=https://www.kernel.org/pub/software/scm/git/git-$VERSION.tar.xz

HOST=x86_64-elf-redox

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
    if [ ! -d zlib ]
    then
	mkdir zlib
    	if [ ! -f zlib-1.2.11.tar.gz ]
	then
            wget http://zlib.net/zlib-1.2.11.tar.gz
        fi
	tar xvf zlib-1.2.11.tar.gz -C zlib --strip-components 1
    fi

    rm -rf zlib-prefix
    mkdir zlib-prefix

    pushd zlib
	./configure --static --prefix=/
	make -j"$(nproc)"
	make DESTDIR="$PWD/../zlib-prefix" install
    popd

    autoconf
    ./configure --host=${HOST} --prefix=/ --with-zlib="${PWD}/zlib-prefix"
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
    make prefix="$dest" install
    ${STRIP} $1/bin/* || true
    ${STRIP} $1/libexec/git-core/* || true
    skip=1
}
