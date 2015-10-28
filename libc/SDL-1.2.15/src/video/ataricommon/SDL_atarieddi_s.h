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

#ifndef _SDL_Atari_eddi_s_h
#define _SDL_Atari_eddi_s_h

/*--- Defines ---*/

/* EdDI versions */

#define EDDI_10	(0x0100)
#define EDDI_11 (0x0110)

/* Screen format */

enum {
	VDI_FORMAT_UNKNOWN=-1,
	VDI_FORMAT_INTER=0,	/* Interleaved bitplanes */
	VDI_FORMAT_VDI=1,	/* VDI independent */
	VDI_FORMAT_PACK=2	/* Packed pixels */
};

/* CLUT types */
enum {
	VDI_CLUT_NONE=0,	/* Monochrome mode */
	VDI_CLUT_HARDWARE,	/* <256 colours mode */
	VDI_CLUT_SOFTWARE	/* True colour mode */
};

/*--- Functions ---*/

unsigned long Atari_get_EdDI_version(void *function_pointer);

#endif /* _SDL_Atari_eddi_s_h */
