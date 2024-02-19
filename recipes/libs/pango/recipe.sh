VERSION="1.42.4"
TAR="ftp.gnome.org/pub/GNOME/sources/pango/${VERSION%.*}/pango-${VERSION}.tar.xz"
BUILD_DEPENDS=(cairo expat fontconfig freetype2 fribidi gettext glib harfbuzz libffi libiconv libpng pcre pixman zlib)

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
