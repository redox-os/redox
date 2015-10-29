#!/bin/bash
set -e
./lua.sh $*
./zlib.sh $*
./freetype.sh $*
./libpng.sh $*
./libiconv.sh $*
./sdl.sh $*
./sdl_gfx.sh $*
./sdl_image.sh $*
./sdl_ttf.sh $*
./dosbox.sh $*
./freeciv.sh $*
