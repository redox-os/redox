VERSION="1.16.0"
TAR=https://www.cairographics.org/releases/cairo-$VERSION.tar.xz
BUILD_DEPENDS=(zlib pixman freetype libpng)

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_update {
	echo "skipping update"
	skip=1
}

function recipe_build {
	#Workaround to disable the not redox compatible tests
	printf "all:\n\ninstall:\n" > ./test/Makefile.in
	printf "all:\n\ninstall:\n" > ./perf/Makefile.in

	sysroot="$(realpath ../sysroot)"
	export LDFLAGS="-L$sysroot/lib"
	export CPPFLAGS="-I$sysroot/include"
	CFLAGS="-DCAIRO_NO_MUTEX=1" ./configure --host=${HOST} --prefix=/ --enable-xlib=no --enable-script=no --enable-interpreter=no
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
	echo "skipping stage"
	dest="$(realpath $1)"
	make DESTDIR="$dest" install
	skip=1
}
