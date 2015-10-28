#!/bin/bash
set -e
./lua.sh $*
./zlib.sh $*
./libpng.sh $*
./freetype.sh $*
./sdl.sh $*
./sdl_gfx.sh $*
./sdl_image.sh $*
./sdl_ttf.sh $*
