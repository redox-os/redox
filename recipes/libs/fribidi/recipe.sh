VERSION="1.0.13"
TAR="https://github.com/fribidi/fribidi/releases/download/v${VERSION}/fribidi-${VERSION}.tar.xz"
BUILD_DEPENDS=()

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_build {
	sysroot="$(realpath ../sysroot)"
	export CFLAGS="-I$sysroot/include"
	export LDFLAGS="-L$sysroot/lib --static"
	./configure \
	    --build=${BUILD} \
	    --host=${HOST} \
	    --prefix=/ \
	    --disable-shared \
	    --enable-static
	sed -i 's|#define HAVE_SYS_TIMES_H 1|/* #undef HAVE_SYS_TIMES_H */|g' config.h
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
