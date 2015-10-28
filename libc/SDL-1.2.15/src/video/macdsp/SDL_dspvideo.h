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

#ifndef _SDL_dspvideo_h
#define _SDL_dspvideo_h

#if TARGET_API_MAC_OSX
#  include <DrawSprocket/DrawSprocket.h> /* Drawsprocket.framework */
#else
#include <DrawSprocket.h>
#endif

#include "../maccommon/SDL_lowvideo.h"

/* DrawSprocket specific information */
struct DSpInfo {
	DSpContextReference dsp_context;
	CGrafPtr            dsp_back_buffer;
   int                 dsp_old_depth;
   
	/* Flags for hw acceleration */
	int dsp_vram_available;
	int dsp_agp_available;
	
	
}; 
/* Old variable names */
#define dsp_context (this->hidden->dspinfo->dsp_context)
#define dsp_back_buffer (this->hidden->dspinfo->dsp_back_buffer)
#define dsp_old_depth   (this->hidden->dspinfo->dsp_old_depth)
#define dsp_vram_available (this->hidden->dspinfo->dsp_vram_available)
#define dsp_agp_available (this->hidden->dspinfo->dsp_agp_available)

#endif /* _SDL_dspvideo_h */
