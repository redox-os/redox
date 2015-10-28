#!/bin/sh

rm -rf /usr/src/RPM/BUILD/SDL_gfx-2.0.13
(cd ..; tar cvzf SDL_gfx-2.0.13.tar.gz SDL_gfx-2.0.13)
cp ../SDL_gfx-2.0.13.tar.gz /usr/src/RPM/SOURCES/SDL_gfx-2.0.13.tar.gz
rpm -ba SDL_gfx.spec
