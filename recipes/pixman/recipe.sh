VERSION="0.42.2"
TAR=https://www.cairographics.org/releases/pixman-$VERSION.tar.gz

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_build {
	sysroot="$(realpath ../sysroot)"
	./configure \
            --build=${BUILD} \
            --host=${HOST} \
            --prefix=/ \
            --disable-shared \
            --enable-static
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
	rm -f "$dest/lib/"*.la
	skip=1
}
