/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

#include "../macrom/SDL_romvideo.h"

/* Functions to be exported */
extern void Mac_SetCaption(_THIS, const char *title, const char *icon);

/*
 * There's no Carbonized gamma support in Mac OS X, since PBStatusSync() and
 *  Control() aren't supported in OS X's Carbonlib. Use the Quartz driver
 *  instead.
 */
#define SDL_MACCLASSIC_GAMMA_SUPPORT ((defined(__APPLE__) && defined(__MACH__)) == 0)

#if SDL_MACCLASSIC_GAMMA_SUPPORT
extern void Mac_QuitGamma(_THIS);
extern int Mac_SetGammaRamp(_THIS, Uint16 *ramp);
extern int Mac_GetGammaRamp(_THIS, Uint16 *ramp);
#endif

