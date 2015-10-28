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

/*
 *	GEM SDL video driver implementation
 *	Window manager functions
 * 
 *	Patrice Mandin
 */

#include "SDL_gemvideo.h"

/* Functions prototypes */
extern void GEM_SetCaption(_THIS, const char *title, const char *icon);
extern void GEM_SetIcon(_THIS, SDL_Surface *icon, Uint8 *mask);
extern int GEM_IconifyWindow(_THIS);
extern SDL_GrabMode GEM_GrabInput(_THIS, SDL_GrabMode mode);
