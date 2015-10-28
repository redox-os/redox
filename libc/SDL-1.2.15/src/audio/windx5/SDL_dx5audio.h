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

#ifndef _SDL_lowaudio_h
#define _SDL_lowaudio_h

#include "directx.h"

#include "../SDL_sysaudio.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_AudioDevice *this

/* The DirectSound objects */
struct SDL_PrivateAudioData {
	LPDIRECTSOUND sound;
	LPDIRECTSOUNDBUFFER mixbuf;
	int NUM_BUFFERS;
	int mixlen, silence;
	DWORD lastchunk;
	Uint8 *locked_buf;
	HANDLE audio_event;
};

/* Old variable names */
#define sound			(this->hidden->sound)
#define mixbuf			(this->hidden->mixbuf)
#define NUM_BUFFERS		(this->hidden->NUM_BUFFERS)
#define mixlen			(this->hidden->mixlen)
#define silence			(this->hidden->silence)
#define lastchunk		(this->hidden->lastchunk)
#define locked_buf		(this->hidden->locked_buf)
#define audio_event		(this->hidden->audio_event)

#endif /* _SDL_lowaudio_h */
