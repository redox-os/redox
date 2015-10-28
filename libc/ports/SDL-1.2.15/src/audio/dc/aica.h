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

#ifndef _AICA_H_
#define _AICA_H_

#define	AICA_MEM	0xa0800000

#define SM_8BIT		1
#define SM_16BIT	0
#define SM_ADPCM	2

void aica_play(int ch,int mode,unsigned long smpptr,int looptst,int loopend,int freq,int vol,int pan,int loopflag);
void aica_stop(int ch);
void aica_vol(int ch,int vol);
void aica_pan(int ch,int pan);
void aica_freq(int ch,int freq);
int aica_get_pos(int ch);

#endif
