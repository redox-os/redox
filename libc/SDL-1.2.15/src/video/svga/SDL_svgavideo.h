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

#ifndef _SDL_svgavideo_h
#define _SDL_svgavideo_h

#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

/* Private display data */
struct SDL_PrivateVideoData {
#define NUM_MODELISTS	4		/* 8, 16, 24, and 32 bits-per-pixel */
	int SDL_nummodes[NUM_MODELISTS];
	SDL_Rect **SDL_modelist[NUM_MODELISTS];
	int *SDL_vgamode[NUM_MODELISTS];

	/* information for double-buffering */
	int flip_page;
	int flip_offset[2];
	Uint8 *flip_address[2];

	/* Set to 1 if we're in banked video mode */
	int banked;
};
/* Old variable names */
#define SDL_nummodes		(this->hidden->SDL_nummodes)
#define SDL_modelist		(this->hidden->SDL_modelist)
#define SDL_vgamode		(this->hidden->SDL_vgamode)
#define flip_page		(this->hidden->flip_page)
#define flip_offset		(this->hidden->flip_offset)
#define flip_address		(this->hidden->flip_address)
#define	banked			(this->hidden->banked)

#endif /* _SDL_svgavideo_h */

