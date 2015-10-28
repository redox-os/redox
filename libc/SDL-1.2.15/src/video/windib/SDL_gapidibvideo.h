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

#ifndef _SDL_gapidibvideo_h
#define _SDL_gapidibvideo_h

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

/* typedef these to be able to define pointers, but still force everybody who
 * wants to access them to include the corresponding header */
typedef struct GapiInfo GapiInfo;
typedef struct DibInfo DibInfo;

/* for PDA */
typedef enum
{
	SDL_ORIENTATION_UP,
	SDL_ORIENTATION_DOWN,
	SDL_ORIENTATION_LEFT,
	SDL_ORIENTATION_RIGHT
} SDL_ScreenOrientation;

/* Private display data shared by gapi and windib*/
struct SDL_PrivateVideoData {
	int supportRotation; /* for Pocket PC devices */
	DWORD origRotation; /* for Pocket PC devices */
	
	GapiInfo* gapiInfo;
	DibInfo* dibInfo;
};

#endif
