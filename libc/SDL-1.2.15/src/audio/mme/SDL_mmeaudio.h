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

/* Allow access to a raw mixing buffer */

#ifndef _SDL_lowaudio_h
#define _SDL_lowaudio_h

#include "../SDL_sysaudio.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_AudioDevice *this
#define NUM_BUFFERS 2

struct SharedMem {
    HWAVEOUT sound;
    WAVEHDR wHdr[NUM_BUFFERS];
    PCMWAVEFORMAT wFmt;
};

struct SDL_PrivateAudioData {
    Uint8 *mixbuf;          /* The raw allocated mixing buffer */
    struct SharedMem *shm;
    int next_buffer;
};

#define shm			(this->hidden->shm)
#define mixbuf			(this->hidden->mixbuf)
#define next_buffer		(this->hidden->next_buffer)
/* Old variable names */
#endif /* _SDL_lowaudio_h */
