VERSION="2.13.91"
TAR="https://www.freedesktop.org/software/fontconfig/release/fontconfig-${VERSION}.tar.xz"
BUILD_DEPENDS=(expat freetype libpng zlib)

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_update {
	echo "skipping update"
	skip=1
}

function recipe_build {
	wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
	sysroot="$(realpath ../sysroot)"
	export CFLAGS="-I$sysroot/include"
	export LDFLAGS="-L$sysroot/lib --static"
	./configure \
	    --build=${BUILD} \
	    --host=${HOST} \
	    --prefix=/ \
	    --disable-shared \
	    --enable-static \
	    ac_cv_func_XML_SetDoctypeDeclHandler=yes
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
