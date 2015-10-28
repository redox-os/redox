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
	m68k assembly mix routines

	Patrice Mandin
*/

#if (defined(__m68k__) && !defined(__mcoldfire__)) && defined(__GNUC__)
void SDL_MixAudio_m68k_U8(char* dst,char* src, long len, long volume, char* mix8);
void SDL_MixAudio_m68k_S8(char* dst,char* src, long len, long volume);

void SDL_MixAudio_m68k_S16MSB(short* dst,short* src, long len, long volume);
void SDL_MixAudio_m68k_S16LSB(short* dst,short* src, long len, long volume);
#endif
