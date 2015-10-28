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

#ifndef _SDL_dibvideo_h
#define _SDL_dibvideo_h

#define WIN32_LEAN_AND_MEAN
#include <windows.h>


/* Private display data */
struct DibInfo {
	HBITMAP screen_bmp;
    HPALETTE screen_pal;
    LOGPALETTE *screen_logpal;
    BOOL grab_palette;

#define NUM_MODELISTS	4		/* 8, 16, 24, and 32 bits-per-pixel */
    int SDL_nummodes[NUM_MODELISTS];
    SDL_Rect **SDL_modelist[NUM_MODELISTS];
        
#ifdef _WIN32_WCE
	int supportRotation; /* for Pocket PC devices */
	DWORD origRotation; /* for Pocket PC devices */
#endif

    /* Screensaver settings */
    int allow_screensaver;
};
/* Old variable names */
#define screen_bmp		(this->hidden->dibInfo->screen_bmp)
#define screen_pal		(this->hidden->dibInfo->screen_pal)
#define screen_logpal		(this->hidden->dibInfo->screen_logpal)
#define grab_palette		(this->hidden->dibInfo->grab_palette)
#define SDL_nummodes		(this->hidden->dibInfo->SDL_nummodes)
#define SDL_modelist		(this->hidden->dibInfo->SDL_modelist)
#define allow_screensaver	(this->hidden->dibInfo->allow_screensaver)

#endif /* _SDL_dibvideo_h */
