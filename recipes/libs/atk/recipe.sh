VERSION="2.26.1"
TAR="https://ftp.gnome.org/pub/gnome/sources/atk/${VERSION%.*}/atk-${VERSION}.tar.xz"
BUILD_DEPENDS=(gettext glib libffi libiconv pcre)

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_build {
	wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
	sysroot="$(realpath ../sysroot)"
	export CFLAGS="-I$sysroot/include"
	export LDFLAGS="-L$sysroot/lib --static"
	export GLIB_MKENUMS="$sysroot/bin/glib-mkenums"
	export GLIB_GENMARSHAL="$sysroot/bin/glib-genmarshal"
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
