BUILD_DEPENDS=(sdl2_image sdl2_mixer sdl2_ttf sdl2 liborbital llvm mesa mesa_glu freetype libjpeg libpng zlib)

function recipe_version {
    printf "1.0.0"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_prepare {
    rm -rf source
    mkdir source
    cp gears.c source
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    "${CXX}" -O2 -I "$sysroot/include" -L "$sysroot/lib" gears.c -o sdl2_gears -static -lSDL2_image -lSDL2_mixer -lSDL2_ttf -lSDL2 -lorbital $("${PKG_CONFIG}" --libs glu) -lfreetype -lpng -ljpeg -lglapi -lz
    set +x
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
    mkdir -pv "$dest/games/sdl2_gears"
    mkdir -pv "$dest/home/user"
    cp -v "sdl2_gears" "$dest/games/sdl2_gears/sdl2_gears"
    cp -v "../image.png" "$dest/home/user/image.png"
    cp -v "../music.wav" "$dest/home/user/music.wav"
    cp -v "../font.ttf" "$dest/home/user/font.ttf"
    skip=1
}
