/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/*
	MCSN control structure

	Patrice Mandin
*/

#ifndef _SDL_mintaudio_mcsh_h
#define _SDL_mintaudio_mcsh_h

typedef struct {
	unsigned short version;	/* Version */
	unsigned short size;	/* Size of structure */

	unsigned short play;	/* Replay capability */
	unsigned short record;	/* Record capability */
	unsigned short dsp;		/* DSP56K present */
	unsigned short pint;	/* Interrupt at end of replay */
	unsigned short rint;	/* Interrupt at end of record */

	unsigned long res1;		/* Frequency of external clock */
	unsigned long res2;
	unsigned long res3;
	unsigned long res4;
} cookie_mcsn_t;

enum {
	MCSN_ST=0,
	MCSN_TT,
	MCSN_STE=MCSN_TT,
	MCSN_FALCON,
	MCSN_MAC=MCSN_FALCON
};

#define SETSMPFREQ	7	/* Set sample frequency */

#endif /* _SDL_mintaudio_mcsh_h */
