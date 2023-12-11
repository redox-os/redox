BUILD_DEPENDS=(sdl2-image sdl2-mixer sdl2-ttf sdl2 liborbital llvm mesa freetype2 libjpeg libpng libogg libvorbis zlib)

function recipe_version {
    printf "1.0.0"
    skip=1
}

function recipe_prepare {
    rm -rf source
    mkdir source
    cp gears.c source
    mkdir source/assets
    cp assets/* source/assets
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    "${CXX}" -O2 -I "$sysroot/include" -L "$sysroot/lib" gears.c -o sdl2_gears -static -lSDL2_image -lSDL2_mixer -lSDL2_ttf -lSDL2 -lorbital $("${PKG_CONFIG}" --libs osmesa) -lfreetype -lpng -ljpeg -lvorbisfile -lvorbis -logg -lz
    set +x
    skip=1
}

function recipe_clean {
    echo "skipping clean"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/games/sdl2_gears"
    mkdir -pv "$dest/games/sdl2_gears/assets"
    cp -v "sdl2_gears" "$dest/games/sdl2_gears/sdl2_gears"
    cp -v "assets/image.png" "$dest/games/sdl2_gears/assets/image.png"
    cp -v "assets/music.wav" "$dest/games/sdl2_gears/assets/music.wav"
    cp -v "assets/font.ttf" "$dest/games/sdl2_gears/assets/font.ttf"
    skip=1
}
