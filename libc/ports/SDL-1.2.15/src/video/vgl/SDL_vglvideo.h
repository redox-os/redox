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

#ifndef _SDL_vglvideo_h
#define _SDL_vglvideo_h

#include <sys/fbio.h>
#include <sys/consio.h>
#include <vgl.h>

#include "SDL_mouse.h"
#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

typedef struct {
	int ModeId;
	int Depth;
	int Rmask;
	int Gmask;
	int Bmask;
	VGLBitmap ModeInfo;
} VGLMode;

/* Private display data */
struct SDL_PrivateVideoData {
#define NUM_MODELISTS	4		/* 8, 16, 24, and 32 bits-per-pixel */
	int SDL_nummodes[NUM_MODELISTS];
	SDL_Rect **SDL_modelist[NUM_MODELISTS];
	SDL_mutex *hw_lock;
	VGLMode *VGLCurMode;
	int flip_page;
	byte *flip_address[2];
};
/* Old variable names */
#define SDL_nummodes	(this->hidden->SDL_nummodes)
#define SDL_modelist	(this->hidden->SDL_modelist)
#define hw_lock		(this->hidden->hw_lock)
#define VGLCurMode	(this->hidden->VGLCurMode)
#define flip_page	(this->hidden->flip_page)
#define flip_address	(this->hidden->flip_address)

#endif /* _SDL_vglvideo_h */
