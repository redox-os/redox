VERSION="0.36.0"
TAR=https://www.cairographics.org/releases/pixman-$VERSION.tar.gz

function recipe_version {
	echo "$VERSION"
	skip=1
}

function recipe_update {
	echo "skipping update"
	skip=1
}

function recipe_build {
	sysroot="$(realpath ../sysroot)"
	./configure \
		--host=${HOST} \
		--prefix=/ \
		--disable-shared \
		--enable-static
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
	echo "skipping stage"
	dest="$(realpath $1)"
	make DESTDIR="$dest" install
	skip=1
}
